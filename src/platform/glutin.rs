use glutin::{ContextWrapper, PossiblyCurrent};
use log::Level;
use winit::event_loop::EventLoopWindowTarget;
use winit::window::{Window, WindowBuilder};
use crate::opengl::Context;

pub fn setup_logger(level: Level) {
    env_logger::builder()
        .filter_level(level.to_level_filter())
        .format_timestamp(None)
        .format_target(false)
        .init()
}

pub trait WindowBuilderExt {
    fn build_context<T>(self, el: &EventLoopWindowTarget<T>) -> (ContextWrapper<PossiblyCurrent, Window>, Context);
}

impl WindowBuilderExt for WindowBuilder {

    fn build_context<T>(self, el: &EventLoopWindowTarget<T>) -> (ContextWrapper<PossiblyCurrent, Window>, Context) {
        unsafe {
            let window = glutin::ContextBuilder::new()
                .with_vsync(true)
                .build_windowed(self, el)
                .expect("Can not get OpenGL context!")
                .make_current()
                .expect("Can set OpenGL context as current!");
            let gl = Context::from_loader_function(|s| window.get_proc_address(s) as *const _);
            (window, gl)
        }
    }

}