use std::fmt::Debug;
use std::mem::take;
use anyhow::Result;
use std::time::Duration;
use glam::Vec2;
use instant::Instant;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::window::{Fullscreen, Window};
use crate::opengl::Context;
use winit::event::Event as WinitEvent;

pub type GlowContext = glow::Context;

pub trait PlatformWindow {
    fn window(&self) -> &Window;
    fn swap_buffers(&self);
    fn resize_surface(&self, size: PhysicalSize<u32>);
}

pub trait Platform {
    type Window: PlatformWindow;
    fn create_context<T>(el: &EventLoopWindowTarget<T>) -> (Self::Window, GlowContext);
}

#[derive(Debug, Copy, Clone)]
pub enum Event {
    WindowResize(f32, f32),
    Click(Vec2),
    DragStart(MouseDelta),
    Drag(MouseDelta),
    DragEnd(MouseDelta),
    Zoom(Vec2, f32),
}

#[derive(Debug, Copy, Clone)]
pub enum Event2 {
    Resize(u32, u32),
}

enum ApplicationState<G: Game2> {
    Active(G, Context),
    Suspended(G::Bundle),
    Invalid
}

impl<G: Game2> Default for ApplicationState<G> {
    fn default() -> Self {
        Self::Invalid
    }
}

pub struct AppContext<'a> {
    gl: &'a Context,
    screen_size: (u32, u32)
}

impl<'a> AppContext<'a> {
    fn new(gl: &'a Context, screen_size: (u32, u32)) -> Self {
        Self { gl, screen_size }
    }

    pub fn gl(&self) -> &'a Context {
        self.gl
    }

    pub fn screen_width(&self) -> u32 {
        self.screen_size.0
    }

    pub fn screen_height(&self) -> u32 {
        self.screen_size.0
    }
}

pub struct Application<G: Game2> {
    state: ApplicationState<G>,
    screen_size: (u32, u32)
}

impl<G: Game2> Application<G> {
    pub fn new() -> Result<Self> {
        let bundle = G::Bundle::new()?;
        Ok(Self {
            state: ApplicationState::Suspended(bundle),
            screen_size: (100, 100)
        })
    }

    pub fn resume(&mut self, gl: GlowContext, screen_size: impl Into<Option<(u32, u32)>>) {
        if let Some(new_size) = screen_size.into() {
            self.screen_size = new_size;
        }
        self.state = match take(&mut self.state) {
            ApplicationState::Suspended(bundle) => {
                let ctx = Context::from_glow(gl);
                match G::resume(AppContext::new(&ctx, self.screen_size), bundle.clone()) {
                    Ok(active) => ApplicationState::Active(active, ctx),
                    Err(err) => {
                        log::error!("Can't resume application:\n{}", err);
                        ApplicationState::Suspended(bundle)
                    }
                }
            },
            state => state
        }
    }

    pub fn suspend(&mut self) {
        self.state = match take(&mut self.state) {
            ApplicationState::Active(active, _) => ApplicationState::Suspended(active.suspend()),
            state => state
        }
    }

    pub fn get_screen_size(&self) -> (u32, u32) {
        self.screen_size
    }

    pub fn set_screen_size(&mut self, screen_size: (u32, u32)) {
        self.screen_size = screen_size;
        self.call_event(Event2::Resize(screen_size.0, screen_size.1));
    }

    fn call_event(&mut self, event: Event2) {
        if let ApplicationState::Active(game, gl) = &mut self.state {
            game.event(AppContext::new(gl, self.screen_size), event);
        }
    }

}

pub trait Bundle: Clone + Sized {
    fn new() -> Result<Self>;
}

pub trait Game2: Sized {
    type Bundle: Bundle;

    fn resume(ctx: AppContext, bundle: Self::Bundle) -> Result<Self>;
    fn suspend(self) -> Self::Bundle;

    fn draw(&mut self, ctx: AppContext);
    fn event(&mut self, ctx: AppContext, event: Event2);
}

pub trait Game: 'static + Sized {
    fn initialize(ctx: &Context) -> Self;
    fn draw(&mut self, ctx: &Context, delta: Duration);
    fn event(&mut self, ctx: &Context, event: Event);

    fn run<P: Platform + 'static>() -> ! {
        let event_loop = EventLoop::new();
        let (window, ctx) = P::create_context(&event_loop);
        let ctx = Context::from_glow(ctx);

        let mut handler = Self::initialize(&ctx);


        let mut last_update = Instant::now();
        let mut mouse_tracker = MouseTracker::new();
        let mut dragging = false;

        event_loop.run(move |event, _, control_flow| match event {
            WinitEvent::WindowEvent { event, window_id, } if window_id == window.window().id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    ctx.viewport(0, 0, size.width as i32, size.height as i32);
                    handler.event(&ctx, Event::WindowResize(size.width as f32, size.height as f32));
                    window.resize_surface(size);
                },
                WindowEvent::MouseWheel { delta, .. } => {
                    let dy = match delta {
                        MouseScrollDelta::LineDelta(_, dy) => dy,
                        MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0
                    };
                    handler.event(&ctx, Event::Zoom(mouse_tracker.position(), dy))
                },
                WindowEvent::CursorMoved {position, .. } => {
                    let size = window.window().inner_size();
                    mouse_tracker.update_position(Vec2::new(position.x as f32 / size.width as f32, 1.0 - position.y as f32 / size.height as f32));

                    if dragging {
                        handler.event(&ctx, Event::Drag(mouse_tracker.delta()))
                    }
                }
                WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                    handler.event(&ctx, Event::Click(mouse_tracker.position()))
                },
                WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Middle, .. } => {
                    dragging = true;
                    handler.event(&ctx, Event::DragStart(mouse_tracker.delta()))
                },
                WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Middle, .. } => {
                    dragging = false;
                    handler.event(&ctx, Event::DragEnd(mouse_tracker.delta()))
                },
                WindowEvent::KeyboardInput { input: KeyboardInput { state: ElementState::Pressed, virtual_keycode: Some(VirtualKeyCode::F11), .. }, .. } => {
                    match window.window().fullscreen() {
                        None => window.window().set_fullscreen(Some(Fullscreen::Borderless(None))),
                        Some(_) => window.window().set_fullscreen(None)
                    }
                }
                _ => {}
            },
            WinitEvent::RedrawRequested(window_id) if window_id == window.window().id() => {
                let now = Instant::now();
                handler.draw(&ctx, now - last_update);
                last_update = now;
                *control_flow = ControlFlow::Poll;
                window.swap_buffers();
            },
            WinitEvent::MainEventsCleared => window.window().request_redraw(),
            WinitEvent::LoopDestroyed => {},
            _ => {}
        });
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MouseDelta(pub Vec2, pub Duration);

impl MouseDelta {

    pub fn absolute(self) -> Vec2 {
        self.0
    }

    #[allow(dead_code)]
    pub fn velocity(self) -> Vec2 {
        self.0 / self.1.as_secs_f32()
    }

}

struct MouseTracker {
    current_position: (Vec2, Instant),
    previous_position: (Vec2, Instant)
}

impl MouseTracker {

    fn new() -> Self {
        Self {
            current_position: (Vec2::ZERO, Instant::now()),
            previous_position: (Vec2::ZERO, Instant::now())
        }
    }

    fn position(&self) -> Vec2 {
        self.current_position.0
    }

    fn delta(&self) -> MouseDelta {
        MouseDelta(
            self.current_position.0 - self.previous_position.0,
            self.current_position.1.saturating_duration_since(self.previous_position.1)
        )
    }

    fn update_position(&mut self, position: Vec2) {
        self.previous_position = self.current_position;
        self.current_position = (position, Instant::now());
    }

}