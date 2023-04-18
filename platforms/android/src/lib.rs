use std::num::NonZeroU32;
use std::ops::Deref;

use android_activity::AndroidApp;
use glutin::config::{Config, ConfigSurfaceTypes, ConfigTemplateBuilder, GlConfig};
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentGlContextSurfaceAccessor, PossiblyCurrentContext};
use glutin::display::Display;
use glutin::display::GlDisplay;
use glutin::surface::{GlSurface, Surface, SurfaceAttributesBuilder, WindowSurface};
use infinity_loop::InfinityLoop;
use infinity_loop::export::{Context, AppContext, Application, Result, GlowContext};
use log::LevelFilter;
use raw_window_handle::{HasRawDisplayHandle, HasRawWindowHandle};
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{WindowEvent, Event, ElementState, MouseButton, MouseScrollDelta, TouchPhase, Touch};
use winit::event_loop::{EventLoopBuilder, ControlFlow, EventLoopWindowTarget};
use winit::platform::android::EventLoopBuilderExtAndroid;
use winit::window::{Window, WindowBuilder};

use crate::android::enable_immersive;

mod android;

struct GlutinContext {
    window: Window,
    _display: Display,
    surface: Surface<WindowSurface>,
    context: PossiblyCurrentContext,
    gl: Context
}

impl Deref for GlutinContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        self.gl()
    }
}

impl AppContext for GlutinContext {
    fn gl(&self) -> &Context {
        &self.gl
    }

    fn screen_size(&self) -> (u32, u32) {
        let size = self.window.inner_size();
        (size.width, size.height)
    }

}

impl GlutinContext {
    
    fn new(event_loop: &EventLoopWindowTarget<()>) -> Result<Self> {
        let window = WindowBuilder::new()
            .build(event_loop)?;
        let display = unsafe { 
            Display::new(event_loop.raw_display_handle(), glutin::display::DisplayApiPreference::Egl)?
        };
        let template = ConfigTemplateBuilder::new()
            .with_alpha_size(8)
            .with_depth_size(0)
            .with_stencil_size(0)
            .compatible_with_native_window(window.raw_window_handle())
            .with_surface_type(ConfigSurfaceTypes::WINDOW)
            .build();
        let config = unsafe {
            display
                .find_configs(template)?
                .reduce(|accum, config| [accum, config]
                        .into_iter()
                        .max_by_key(Config::num_samples)
                        .unwrap())
                .unwrap()
        };
        log::trace!("Picked a config with {} samples", config.num_samples());

        let (width, height): (u32, u32) = window.inner_size().into();
        let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
            window.raw_window_handle(),
            NonZeroU32::new(width).unwrap(),
            NonZeroU32::new(height).unwrap(),
        );
        let surface = unsafe {
            display
                .create_window_surface(&config, &attrs)?
        };
        let context = {
            let context_attributes = ContextAttributesBuilder::new()
                .build(Some(window.raw_window_handle()));

            let fallback_context_attributes = ContextAttributesBuilder::new()
                .with_context_api(ContextApi::Gles(None))
                .build(Some(window.raw_window_handle()));
            let not_current_context = unsafe {
                display
                    .create_context(&config, &context_attributes)
                    .unwrap_or_else(|_| {
                        display
                            .create_context(&config, &fallback_context_attributes)
                            .expect("failed to create context")
                    })
            };
            not_current_context.make_current(&surface)?
        };

        let gl = unsafe {
            GlowContext::from_loader_function(|s| {
                let s = std::ffi::CString::new(s)
                    .expect("failed to construct C string from string for gl proc address");
                display.get_proc_address(&s)
            })
        };

        Ok(Self {
            window,
            _display: display,
            surface,
            context,
            gl: Context::from_glow(gl),
        })
    }

    fn resize(&self, size: PhysicalSize<u32>) {
        self.surface.resize(
            &self.context,
            size.width.try_into().unwrap(),
            size.height.try_into().unwrap(),
        );
    }

    fn swap_buffers(&self) -> bool {
        self.surface.swap_buffers(&self.context)
            .map_err(|err| log::error!("Failed to swap buffers after render: {}", err))
            .is_ok()
    }

    fn request_redraw(&self) {
        self.window.request_redraw()
    }

}

#[no_mangle]
fn android_main(app: AndroidApp) {
   
    android_logger::init_once(android_logger::Config::default()
        .with_max_level(LevelFilter::Trace)
        .with_tag("infinity_loop"));

    log::info!("sdk version: {}", app.config().sdk_version());
    enable_immersive().unwrap();
    //if app.config().sdk_version() >= 31 {
    //    let window = app
    //        .native_window()
    //        .unwrap()
    //        .ptr()
    //        .as_ptr() as _;
    //    let result = unsafe {
    //        let compatibity = ANativeWindow_FrameRateCompatibility::ANATIVEWINDOW_FRAME_RATE_COMPATIBILITY_DEFAULT.0;
    //        let strategy = ANativeWindow_ChangeFrameRateStrategy::ANATIVEWINDOW_CHANGE_FRAME_RATE_ALWAYS.0;
    //        ANativeWindow_setFrameRateWithChangeStrategy(window, 120.0, compatibity as i8, strategy as i8)
    //    };
    //    log::info!("Framerate change: {}", result);
    //}
    

    let event_loop = EventLoopBuilder::new()
        .with_android_app(app)
        .build();

    log::trace!("Loading applicaiton...");
    let mut app = Application::<InfinityLoop, GlutinContext>::new(None).unwrap();

    log::trace!("Running mainloop...");

    let mut pos = PhysicalPosition::new(0.0, 0.0);
    let mut down = false;
    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = match app.should_redraw() {
            true => ControlFlow::Poll,
            false => ControlFlow::Wait
        };
        match event {
            Event::WindowEvent { event, ..} => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    app.with_ctx(|ctx| ctx.resize(size));
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
                WindowEvent::MouseWheel { delta, .. } => {
                    let dy = match delta {
                        MouseScrollDelta::LineDelta(_, dy) => dy,
                        MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0
                    };
                    app.on_mouse_wheel(pos.x as f32, pos.y as f32, dy)
                },
                WindowEvent::Touch(Touch{ phase, location, id, .. }) => match phase {
                    TouchPhase::Started => app.on_press(location.x as f32, location.y as f32, id),
                    TouchPhase::Moved => app.on_move(location.x as f32, location.y as f32, id),
                    TouchPhase::Ended => app.on_release(location.x as f32, location.y as f32, id),
                    TouchPhase::Cancelled => app.on_release(location.x as f32, location.y as f32, id)
                },
                _ => {}
            },
            Event::RedrawRequested(_) => {
                app.redraw();
                if !app.with_ctx(|ctx| ctx.swap_buffers()) {
                    log::error!("Corrupted render context...");
                }
            },
            Event::MainEventsCleared =>  {
                if app.should_redraw() {
                    app.with_ctx(|ctx| ctx.request_redraw());
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

/*
use std::fmt::{Display, Formatter};
use std::ops::Deref;
use android_logger::Config;
use glutin::{Api, ContextWrapper, GlRequest, PossiblyCurrent};
use glutin::dpi::PhysicalPosition;
use glutin::event::{ElementState, Event, MouseButton, MouseScrollDelta, Touch, TouchPhase, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use glutin::window::{Window, WindowBuilder};
use log::{Level};
use infinity_loop::export::{AppContext, Application, Context, GlowContext, Result};
use infinity_loop::InfinityLoop;
use crate::android::enable_immersive;

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

    enable_immersive().unwrap();
    let mut app = Application::<InfinityLoop, GlutinContext>::new(None).unwrap();

    let event_loop = EventLoop::new();

    let mut pos = PhysicalPosition::new(0.0, 0.0);
    let mut down = false;
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
                WindowEvent::MouseWheel { delta, .. } => {
                    let dy = match delta {
                        MouseScrollDelta::LineDelta(_, dy) => dy,
                        MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0
                    };
                    app.on_mouse_wheel(pos.x as f32, pos.y as f32, dy)
                },
                WindowEvent::Touch(Touch{ phase, location, id, .. }) => match phase {
                    TouchPhase::Started => app.on_press(location.x as f32, location.y as f32, id),
                    TouchPhase::Moved => app.on_move(location.x as f32, location.y as f32, id),
                    TouchPhase::Ended => app.on_release(location.x as f32, location.y as f32, id),
                    TouchPhase::Cancelled => app.on_release(location.x as f32, location.y as f32, id)
                },
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

*/