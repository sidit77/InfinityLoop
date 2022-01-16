#[cfg_attr(target_arch = "wasm32", path="platform/wasm.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path="platform/glutin.rs")]
mod platform;

use std::time::Duration;
use glam::Vec2;
use log::Level;
use winit::dpi::PhysicalSize;
use winit::event::WindowEvent;
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
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

    event_loop.run(move |event, _, control_flow| match event {
        WinitEvent::WindowEvent { ref event, window_id, } if window_id == window.window().id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(size) => {
                handler.event(Event::WindowResize(size.width as f32, size.height as f32));
                window.resize(*size);
            },
            _ => {}
        },
        WinitEvent::RedrawRequested(window_id) if window_id == window.window().id() => {
            handler.draw(&ctx, Duration::from_millis(10));
            *control_flow = ControlFlow::Poll;
            window.swap_buffers().unwrap();
        }
        WinitEvent::MainEventsCleared => window.window().request_redraw(),
        WinitEvent::LoopDestroyed => return,
        _ => {}
    });
}