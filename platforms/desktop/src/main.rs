#![windows_subsystem = "windows"]

use std::ops::Deref;
use glutin::{ContextWrapper, GlProfile, PossiblyCurrent};
use glutin::dpi::{PhysicalPosition, PhysicalSize};
use glutin::event::{ElementState, Event, MouseButton, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use glutin::window::{Window, WindowBuilder};
use log::{LevelFilter};
use infinity_loop::{GlowContext, InfinityLoop};
use infinity_loop::export::{AppContext, Application, Context};

struct GlutinContext(ContextWrapper<PossiblyCurrent, Window>, Context);

impl GlutinContext {
    fn new(el: &EventLoopWindowTarget<()>) -> Self {
        unsafe {
            let window_builder = WindowBuilder::new()
                .with_inner_size(PhysicalSize::new(1280, 720))
                .with_min_inner_size(PhysicalSize::new(100, 100))
                .with_title("Infinity Loop");
            let window = glutin::ContextBuilder::new()
                .with_vsync(true)
                .with_depth_buffer(0)
                .with_stencil_buffer(0)
                .with_gl_profile(GlProfile::Core)
                .build_windowed(window_builder, el)
                .expect("Can not get OpenGL context!")
                .make_current()
                .expect("Can set OpenGL context as current!");
            let gl = GlowContext::from_loader_function(|s| window.get_proc_address(s) as *const _);
            Self(window, Context::from_glow(gl))
        }
    }
}

impl Deref for GlutinContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        self.gl()
    }
}

impl AppContext for GlutinContext {
    fn gl(&self) -> &Context {
        &self.1
    }

    fn screen_size(&self) -> (u32, u32) {
        let size = self.0.window().inner_size();
        (size.width, size.height)
    }
}

fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .format_timestamp(None)
        .format_target(false)
        .init();

    let mut app = Application::<InfinityLoop, GlutinContext>::new().unwrap();

    let event_loop = EventLoop::new();

    app.resume(|| GlutinContext::new(&event_loop));

    let mut ctx = None;
    let mut pos = PhysicalPosition::new(0.0, 0.0);
    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = match app.should_redraw() {
            true => ControlFlow::Poll,
            false => ControlFlow::Wait
        };
        match event {
            Event::WindowEvent { event, ..} => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    app.with_ctx(|ctx| ctx.0.resize(size));
                    match size.width == 0 || size.height == 0 {
                        true => ctx = app.suspend().or(ctx.take()),
                        false => app.resume(|| ctx.take().unwrap_or_else(|| GlutinContext::new(event_loop)))
                    }
                    app.set_screen_size(size.into())
                },
                WindowEvent::CursorMoved { position,.. }
                    => pos = position,
                WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, ..}
                    => app.on_click(pos.x as f32, pos.y as f32),
                _ => {}
            },
            Event::RedrawRequested(_) => {
                app.redraw();
                //if window.swap_buffers().is_err() {
                //    println!("Corrupted render context...");
                //}
                app.with_ctx(|ctx| ctx.0.swap_buffers().unwrap())
            },
            Event::MainEventsCleared =>  {
                if app.should_redraw() {
                    app.with_ctx(|ctx| ctx.0.window().request_redraw());
                }
            },
            Event::LoopDestroyed => {
                app.suspend();
            },
            _ => {}
        }
    });

}
