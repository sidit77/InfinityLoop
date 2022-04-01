use std::fmt::{Display, Error, Formatter};
use std::ops::Deref;
use std::time::Instant;
use android_logger::Config;
use glam::Vec2;
use glutin::{Api, ContextWrapper, GlRequest, PossiblyCurrent};
use glutin::event::{ElementState, Event, MouseButton, MouseScrollDelta, Touch, TouchPhase, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use glutin::window::{Window, WindowBuilder};
use log::{Level};
use infinity_loop::export::{AppContext, Application, Context, GlowContext, Result};
use infinity_loop::InfinityLoop;

struct GlutinContext(ContextWrapper<PossiblyCurrent, Window>, Context);

impl GlutinContext {
    fn new(el: &EventLoopWindowTarget<()>) -> Result<Self> {
        check_native_window()?;
        unsafe {
            let window_builder = WindowBuilder::new()
                .with_title("Infinity Loop");
            let window = glutin::ContextBuilder::new()
                .with_vsync(true)
                .with_gl(GlRequest::Specific(Api::OpenGlEs, (3, 0)))
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

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
fn main() {
    android_logger::init_once(
        Config::default()
            .with_min_level(Level::Trace)
            .with_tag("infinity_loop")
    );

    let mut app = Application::<InfinityLoop, GlutinContext>::new().unwrap();

    let event_loop = EventLoop::new();

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
                    app.set_screen_size(size.into())
                },
                WindowEvent::Touch(Touch{ phase: TouchPhase::Started, location, .. })
                    => app.on_click(location.x as f32, location.y as f32),
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
            Event::Resumed => {
                app.resume(|| GlutinContext::new(event_loop));
            },
            Event::Suspended => {
                app.suspend();
            }
            Event::LoopDestroyed => {
                app.suspend();
            },
            _ => {}
        }
    });

}

fn check_native_window() -> std::result::Result<(), NoNativeWindow> {
    match ndk_glue::native_window().is_some() {
        true => Ok(()),
        false => Err(NoNativeWindow)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct NoNativeWindow;

impl Display for NoNativeWindow {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Native window not available")
    }
}

impl std::error::Error for NoNativeWindow {}

//fn enable_immersive() {
//    let vm_ptr = ndk_glue::native_activity().vm();
//    let vm = unsafe { jni::JavaVM::from_raw(vm_ptr) }.unwrap();
//    let env = vm.attach_current_thread_permanently().unwrap();
//    let activity = ndk_glue::native_activity().activity();
//    let window = env
//        .call_method(activity, "getWindow", "()Landroid/view/Window;", &[])
//        .unwrap()
//        .l()
//        .unwrap();
//    let view = env
//        .call_method(window, "getDecorView", "()Landroid/view/View;", &[])
//        .unwrap()
//        .l()
//        .unwrap();
//    let view_class = env.find_class("android/view/View").unwrap();
//    let flag_fullscreen = env
//        .get_static_field(view_class, "SYSTEM_UI_FLAG_FULLSCREEN", "I")
//        .unwrap()
//        .i()
//        .unwrap();
//    let flag_hide_navigation = env
//        .get_static_field(view_class, "SYSTEM_UI_FLAG_HIDE_NAVIGATION", "I")
//        .unwrap()
//        .i()
//        .unwrap();
//    let flag_immersive_sticky = env
//        .get_static_field(view_class, "SYSTEM_UI_FLAG_IMMERSIVE_STICKY", "I")
//        .unwrap()
//        .i()
//        .unwrap();
//    let flag = flag_fullscreen | flag_hide_navigation | flag_immersive_sticky;
//    match env.call_method(
//        view,
//        "setSystemUiVisibility",
//        "(I)V",
//        &[jni::objects::JValue::Int(flag)],
//    ) {
//        Err(_) => log::warn!("Failed to enable immersive mode"),
//        Ok(_) => {}
//    }
//    env.exception_clear().unwrap();
//}