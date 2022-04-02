#![windows_subsystem = "windows"]

use std::collections::VecDeque;
use std::ops::Deref;
use glutin::{ContextWrapper, GlProfile, PossiblyCurrent};
use glutin::dpi::{PhysicalPosition, PhysicalSize};
use glutin::event::{ElementState, Event, KeyboardInput, MouseButton, Touch, TouchPhase, VirtualKeyCode, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use glutin::window::{Fullscreen, Window, WindowBuilder};
use log::{LevelFilter};
use infinity_loop::{InfinityLoop};
use infinity_loop::export::{AppContext, Application, Context, GlowContext, Result};

struct GlutinContext(ContextWrapper<PossiblyCurrent, Window>, Context);

impl GlutinContext {
    fn new(el: &EventLoopWindowTarget<()>) -> Result<Self> {
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
                .build_windowed(window_builder, el)?
                .make_current()
                .map_err(|(_, err)| err)?;
            let gl = GlowContext::from_loader_function(|s| window.get_proc_address(s) as *const _);
            Ok(Self(window, Context::from_glow(gl)))
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
    let mut down = false;
    let mut touchStack = VecDeque::new();
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
                        false => app.resume(|| ctx.take().map_or_else(|| GlutinContext::new(event_loop), |ctx| Ok(ctx)))
                    }
                    app.set_screen_size(size.into())
                },
                WindowEvent::CursorMoved { position,.. } => {
                    pos = position;
                    if down {
                        app.on_move(pos.x as f32, pos.y as f32, 0);
                    }

                },
                WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, ..}  => {
                    app.on_press(pos.x as f32, pos.y as f32, 0);
                    down = true;
                },
                WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, ..} => {
                    app.on_release(pos.x as f32, pos.y as f32, 0);
                    down = false;
                },
                WindowEvent::KeyboardInput {  input: KeyboardInput {  state: ElementState::Pressed, virtual_keycode, .. }, .. } => match virtual_keycode{
                    Some(VirtualKeyCode::F11) => app.with_ctx(|ctx| {
                        let window = ctx.0.window();
                        window.set_fullscreen(match window.fullscreen() {
                            None => Some(Fullscreen::Borderless(None)),
                            Some(_) => None
                        })
                    }),
                    Some(VirtualKeyCode::Return) => {
                        app.on_press(pos.x as f32, pos.y as f32, 1 + touchStack.len() as u64);
                        touchStack.push_back(pos);
                    },
                    Some(VirtualKeyCode::Back) => {
                        if let Some(pos) = touchStack.pop_back() {
                            app.on_release(pos.x as f32, pos.y as f32, 1 + touchStack.len() as u64);
                        }
                    }
                    _ => {}
                }
                _ => {}
            },
            Event::RedrawRequested(_) => {
                app.redraw();
                if app.with_ctx(|ctx| ctx.0.swap_buffers().is_err()) {
                    log::error!("Corrupted render context...");
                }
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
