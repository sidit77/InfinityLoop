#![windows_subsystem = "windows"]

use glutin::{ContextWrapper, GlProfile, PossiblyCurrent};
use glutin::dpi::PhysicalSize;
use glutin::event::{Event, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use glutin::window::{Window, WindowBuilder};
use log::LevelFilter;
use infinity_loop::{GlowContext, InfinityLoop};
use infinity_loop::export::{Application, Context};


fn create_context<T>(el: &EventLoopWindowTarget<T>) -> (ContextWrapper<PossiblyCurrent, Window>, GlowContext) {
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
        (window, gl)
    }
}


fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .format_timestamp(None)
        .format_target(false)
        .init();

    let mut app = Application::<InfinityLoop>::new().unwrap();

    let event_loop = EventLoop::new();
    let (window, gl) = create_context(&event_loop);
    let gl = Context::from_glow(gl);

    app.resume(gl.clone(), Some(window.window().inner_size().into()));

    event_loop.run(move |event, _event_loop, control_flow| {
        match event {
            Event::WindowEvent { event, window_id, } if window_id == window.window().id() => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    window.resize(size);
                    match size.width == 0 || size.height == 0 {
                        true => app.suspend(),
                        false => app.resume(gl.clone(), None)
                    }
                    app.set_screen_size(size.into())
                },
                _ => {}
            },
            Event::RedrawRequested(window_id) if window_id == window.window().id() => {
                app.redraw();
                *control_flow = ControlFlow::Poll;
                if window.swap_buffers().is_err() {
                    println!("Corrupted render context...");
                }
            },
            Event::MainEventsCleared =>  {
                window.window().request_redraw()
            },
            Event::LoopDestroyed => app.suspend(),
            _ => {}
        }
    });

}
