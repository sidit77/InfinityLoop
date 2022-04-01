use std::fmt::Debug;
use std::mem::take;
use std::ops::Deref;
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
    Click(Vec2),
}

enum ApplicationState<G: Game2, A: AppContext> {
    Active{
        game: G,
        ctx: A,
        should_redraw: bool
    },
    Suspended(G::Bundle),
    Invalid
}

impl<G: Game2, A: AppContext> Default for ApplicationState<G, A> {
    fn default() -> Self {
        Self::Invalid
    }
}

pub trait AppContext: Deref<Target = Context> {

    fn gl(&self) -> &Context;

    fn screen_size(&self) -> (u32, u32);

    fn screen_width(&self) -> u32 {
        self.screen_size().0
    }

    fn screen_height(&self) -> u32 {
        self.screen_size().1
    }
}


pub struct Application<G: Game2, A: AppContext> {
    state: ApplicationState<G, A>,
    screen_size: (u32, u32)
}

/**
- error handling in events
**/
impl<G: Game2, A: AppContext> Application<G, A> {
    pub fn new() -> Result<Self> {
        let bundle = G::Bundle::new()?;
        Ok(Self {
            state: ApplicationState::Suspended(bundle),
            screen_size: (100, 100)
        })
    }

    pub fn resume(&mut self, ctx_func: impl FnOnce() -> A) {
        self.state = match take(&mut self.state) {
            ApplicationState::Suspended(bundle) => {
                let ctx = ctx_func();
                self.screen_size = ctx.screen_size();
                match G::resume(&ctx, bundle.clone()) {
                    Ok(game) => {
                        log::info!("Resumed app");
                        ApplicationState::Active{
                            game,
                            ctx,
                            should_redraw: true
                        }
                    },
                    Err(err) => {
                        log::error!("Can't resume application:\n{}", err);
                        ApplicationState::Suspended(bundle)
                    }
                }
            },
            state => state
        }
    }

    pub fn suspend(&mut self) -> Option<A> {
        let (ctx, state) = match take(&mut self.state) {
            ApplicationState::Active{ game, ctx, ..} => {
                log::info!("Suspended app");
                (Some(ctx), ApplicationState::Suspended(game.suspend()))
            },
            state => (None, state)
        };
        self.state = state;
        ctx
    }

    pub fn set_screen_size(&mut self, screen_size: (u32, u32)) {
        self.screen_size = screen_size;
        self.call_event(Event2::Resize(screen_size.0, screen_size.1));
    }

    pub fn redraw(&mut self) {
        if let ApplicationState::Active{game, ctx, should_redraw} = &mut self.state {
            *should_redraw = game.draw(ctx)
        }
    }

    pub fn should_redraw(&self) -> bool {
        match self.state {
            ApplicationState::Active { should_redraw, ..} => should_redraw,
            _ => false
        }
    }

    pub fn on_click(&mut self, x: f32, y: f32) {
        let (width, height) = self.screen_size;
        let pos = Vec2::new(x / width as f32, 1.0 - y / height as f32);
        self.call_event(Event2::Click(pos))
    }

    pub fn with_ctx(&self, f: impl FnOnce(&A)) {
        if let ApplicationState::Active{  ctx, ..} = &self.state {
            f(ctx)
        }
    }

    fn call_event(&mut self, event: Event2) {
        if let ApplicationState::Active{ game, ctx, should_redraw} = &mut self.state {
            game.event(ctx, event);
            *should_redraw = true;
        }
    }

}

pub trait Bundle: Clone + Sized {
    fn new() -> Result<Self>;
}

pub trait Game2: Sized {
    type Bundle: Bundle;

    fn resume<A: AppContext>(ctx: &A, bundle: Self::Bundle) -> Result<Self>;
    fn suspend(self) -> Self::Bundle;

    fn draw<A: AppContext>(&mut self, ctx: &A) -> bool;
    fn event<A: AppContext>(&mut self, ctx: &A, event: Event2);
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