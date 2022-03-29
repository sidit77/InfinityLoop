#![windows_subsystem = "windows"]

use glutin::{ContextWrapper, GlProfile, PossiblyCurrent};
use glutin::dpi::PhysicalSize;
use glutin::event_loop::EventLoopWindowTarget;
use glutin::window::{Window, WindowBuilder};
use log::LevelFilter;
use infinity_loop::{Game, GlowContext, InfinityLoop, Platform, PlatformWindow};

struct GlutinWindow(ContextWrapper<PossiblyCurrent, Window>);

impl PlatformWindow for GlutinWindow {
    fn window(&self) -> &Window {
        self.0.window()
    }

    fn swap_buffers(&self) {
        self.0.swap_buffers().unwrap()
    }

    fn resize_surface(&self, size: PhysicalSize<u32>) {
        self.0.resize(size)
    }
}

struct GlutinPlatform;

impl Platform for GlutinPlatform {
    type Window = GlutinWindow;

    fn create_context<T>(el: &EventLoopWindowTarget<T>) -> (Self::Window, GlowContext) {
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
            (GlutinWindow(window), gl)
        }
    }
}


fn main() {
    env_logger::builder()
        .filter_level(LevelFilter::Debug)
        .format_timestamp(None)
        .format_target(false)
        .init();

    InfinityLoop::run::<GlutinPlatform>()
}
