#[cfg_attr(target_arch = "wasm32", path="platform/wasm.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path="platform/glutin.rs")]
mod platform;

use std::mem::take;
use std::time::Duration;
use glam::Vec2;
use instant::Instant;
use log::Level;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, MouseButton, MouseScrollDelta, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};
use crate::opengl::Context;
use winit::event::Event as WinitEvent;
use crate::app::platform::WindowBuilderExt;

#[derive(Debug, Copy, Clone)]
pub enum Event {
    WindowResize(f32, f32),
    Click(Vec2),
    Drag(Vec2),
    DragEnd(Vec2),
    Touch,
    Zoom(Vec2, f32),
}

#[derive(Copy, Clone)]
enum ClickState {
    Click(Vec2),
    Drag(Vec2, Vec2),
    None
}

impl Default for ClickState {
    fn default() -> Self {
        Self::None
    }
}

pub trait EventHandler {
    fn draw(&mut self, ctx: &Context, delta: Duration);
    fn event(&mut self, event: Event);
}

pub fn run<T: EventHandler + 'static>(builder: impl FnOnce(&Context) -> T) -> ! {
    platform::setup_logger(Level::Debug);

    let event_loop = EventLoop::new();
    let (window, ctx) = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_title("Infinity Loop")
        .build_context(&event_loop);

    let mut handler = builder(&ctx);


    let mut last_update = Instant::now();
    let mut last_mouse_pos = Vec2::ZERO;
    let mut click_state = ClickState::None;

    event_loop.run(move |event, _, control_flow| match event {
        WinitEvent::WindowEvent { event, window_id, } if window_id == window.window().id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(size) => {
                ctx.viewport(0, 0, size.width as i32, size.height as i32);
                handler.event(Event::WindowResize(size.width as f32, size.height as f32));
                window.resize(size);
            },
            WindowEvent::CursorMoved {position, .. } => {
                let size = window.window().inner_size();
                let new_pos = Vec2::new(position.x as f32 / size.width as f32, 1.0 - position.y as f32 / size.height as f32);

                match click_state {
                    ClickState::Click(pos) => if pixel_dist(window.window(), new_pos, pos) > 10.0 {
                        click_state = ClickState::Drag(new_pos, new_pos - pos);
                        handler.event(Event::Drag(new_pos - pos));
                    }
                    ClickState::Drag(pos, _) => {
                        click_state = ClickState::Drag(new_pos, new_pos - pos);
                        handler.event(Event::Drag(new_pos - pos))
                    },
                    ClickState::None => {}
                }
                last_mouse_pos = new_pos;
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    MouseScrollDelta::LineDelta(_, dy) => dy,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0
                };
                handler.event(Event::Zoom(last_mouse_pos, dy))
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                click_state = ClickState::Click(last_mouse_pos);
                handler.event(Event::Touch)
            },
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, .. } => {
                match take(&mut click_state) {
                    ClickState::Click(pos) => handler.event(Event::Click(pos)),
                    ClickState::Drag(_, delta) => handler.event(Event::DragEnd(delta)),
                    _ => {}
                }
            },
            _ => {}
        },
        WinitEvent::RedrawRequested(window_id) if window_id == window.window().id() => {
            let now = Instant::now();
            handler.draw(&ctx, now - last_update);
            last_update = now;
            *control_flow = ControlFlow::Poll;
            window.swap_buffers().unwrap();
        },
        WinitEvent::MainEventsCleared => window.window().request_redraw(),
        WinitEvent::LoopDestroyed => return,
        _ => {}
    });
}

fn pixel_dist(window: &Window, p1: Vec2, p2: Vec2) -> f32 {
    let s: Vec2 = Vec2::new(window.inner_size().width as f32, window.inner_size().height as f32);
    let p1 = p1 * s;
    let p2 = p2 * s;
    p1.distance(p2) / window.scale_factor() as f32
}