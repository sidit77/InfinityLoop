#![windows_subsystem = "windows"]

use std::collections::VecDeque;
use std::ops::Deref;
use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextAttributesBuilder, NotCurrentGlContextSurfaceAccessor, PossiblyCurrentContext};
use glutin::display::{Display, GetGlDisplay, GlDisplay};
use glutin::surface::{GlSurface, Surface, WindowSurface};
use glutin_winit::{ApiPrefence, DisplayBuilder, finalize_window};
use winit::dpi::{LogicalSize, PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, Event, KeyboardInput, MouseButton, MouseScrollDelta, Touch, TouchPhase, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::window::{Fullscreen, Window, WindowBuilder};
use log::{LevelFilter};
use raw_window_handle::HasRawWindowHandle;
use infinity_loop::{InfinityLoop};
use infinity_loop::export::{AppContext, Application, Context, GlowContext, Result};

pub struct GlutinWindowContext {
    window: Window,
    gl_context: PossiblyCurrentContext,
    gl_display: Display,
    gl_surface: Surface<WindowSurface>,
}

impl GlutinWindowContext {

    unsafe fn new(event_loop: &EventLoopWindowTarget<()>) -> Self {
        let winit_window_builder = WindowBuilder::new()
            .with_resizable(true)
            .with_inner_size(LogicalSize::new(1280, 720))
            .with_title("Infinity Loop");

        let config_template_builder = ConfigTemplateBuilder::new()
            .prefer_hardware_accelerated(None)
            .with_depth_size(0)
            .with_stencil_size(0)
            .with_transparency(false);

        log::debug!("trying to get gl_config");
        let (mut window, gl_config) =
            DisplayBuilder::new() // let glutin-winit helper crate handle the complex parts of opengl context creation
                .with_preference(ApiPrefence::FallbackEgl) // https://github.com/emilk/egui/issues/2520#issuecomment-1367841150
                .with_window_builder(Some(winit_window_builder.clone()))
                .build(
                    event_loop,
                    config_template_builder,
                    |mut config_iterator| {
                        config_iterator.next().expect(
                            "failed to find a matching configuration for creating glutin config",
                        )
                    },
                )
                .expect("failed to create gl_config");
        let gl_display = gl_config.display();
        log::debug!("found gl_config: {:?}", &gl_config);

        let raw_window_handle = window.as_ref().map(|w| w.raw_window_handle());
        log::debug!("raw window handle: {:?}", raw_window_handle);
        let context_attributes =
            ContextAttributesBuilder::new().build(raw_window_handle);
        // by default, glutin will try to create a core opengl context. but, if it is not available, try to create a gl-es context using this fallback attributes
        let fallback_context_attributes = ContextAttributesBuilder::new()
            .with_context_api(glutin::context::ContextApi::Gles(None))
            .build(raw_window_handle);
        let not_current_gl_context = unsafe {
            gl_display
                .create_context(&gl_config, &context_attributes)
                .unwrap_or_else(|_| {
                    log::debug!("failed to create gl_context with attributes: {:?}. retrying with fallback context attributes: {:?}",
                            &context_attributes,
                            &fallback_context_attributes);
                    gl_config
                        .display()
                        .create_context(&gl_config, &fallback_context_attributes)
                        .expect("failed to create context even with fallback attributes")
                })
        };

        // this is where the window is created, if it has not been created while searching for suitable gl_config
        let window = window.take().unwrap_or_else(|| {
            log::debug!("window doesn't exist yet. creating one now with finalize_window");
            finalize_window(event_loop, winit_window_builder.clone(), &gl_config)
                .expect("failed to finalize glutin window")
        });
        let (width, height): (u32, u32) = window.inner_size().into();
        let width = std::num::NonZeroU32::new(width.max(1)).unwrap();
        let height = std::num::NonZeroU32::new(height.max(1)).unwrap();
        let surface_attributes =
            glutin::surface::SurfaceAttributesBuilder::<WindowSurface>::new()
                .build(window.raw_window_handle(), width, height);
        log::debug!(
            "creating surface with attributes: {:?}",
            &surface_attributes
        );
        let gl_surface = unsafe {
            gl_display
                .create_window_surface(&gl_config, &surface_attributes)
                .unwrap()
        };
        log::debug!("surface created successfully: {gl_surface:?}.making context current");
        let gl_context = not_current_gl_context.make_current(&gl_surface).unwrap();

        gl_surface
            .set_swap_interval(
                &gl_context,
                glutin::surface::SwapInterval::Wait(std::num::NonZeroU32::new(1).unwrap()),
            )
            .unwrap();

        GlutinWindowContext {
            window,
            gl_context,
            gl_display,
            gl_surface,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&self, physical_size: PhysicalSize<u32>) {
        self.gl_surface.resize(
            &self.gl_context,
            physical_size.width.try_into().unwrap(),
            physical_size.height.try_into().unwrap(),
        );
    }

    pub fn swap_buffers(&self) -> glutin::error::Result<()> {
        self.gl_surface.swap_buffers(&self.gl_context)
    }

    fn get_proc_address(&self, addr: &std::ffi::CStr) -> *const std::ffi::c_void {
        self.gl_display.get_proc_address(addr)
    }
}

struct GlutinContext(GlutinWindowContext, Context);

impl GlutinContext {
    fn new(el: &EventLoopWindowTarget<()>) -> Result<Self> {
        let window = unsafe { GlutinWindowContext::new(el) };

        let gl = unsafe {
            GlowContext::from_loader_function(|s| {
                let s = std::ffi::CString::new(s)
                    .expect("failed to construct C string from string for gl proc address");
                window.get_proc_address(&s)
            })
        };
        Ok(Self(window, Context::from_glow(gl)))
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
        .filter_level(LevelFilter::Trace)
        .format_timestamp(None)
        .format_target(false)
        .init();

    let save_file = "save.json";

    let mut app = Application::<InfinityLoop, GlutinContext>::new(std::fs::read_to_string(save_file).ok()).unwrap();

    let event_loop = EventLoop::new();

    app.resume(|| GlutinContext::new(&event_loop));
    assert!(app.is_running());

    let mut ctx = None;
    let mut pos = PhysicalPosition::new(0.0, 0.0);
    let mut down = false;
    let mut touch_stack = VecDeque::new();
    event_loop.run(move |event, event_loop, control_flow| {
        *control_flow = match app.should_redraw() {
            true => ControlFlow::Poll,
            false => app.next_timeout().map_or(ControlFlow::Wait, ControlFlow::WaitUntil)
        };
        match event {
            Event::WindowEvent { event, ..} => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
                    app.with_ctx(|ctx| ctx.0.resize(size));
                    match size.width == 0 || size.height == 0 {
                        true => ctx = app.suspend().or_else(|| ctx.take()),
                        false => app.resume(|| ctx.take().map_or_else(|| GlutinContext::new(event_loop), Ok))
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
                WindowEvent::KeyboardInput {  input: KeyboardInput {  state: ElementState::Pressed, virtual_keycode, .. }, .. } => match virtual_keycode{
                    Some(VirtualKeyCode::F11) => app.with_ctx(|ctx| {
                        let window = ctx.0.window();
                        window.set_fullscreen(match window.fullscreen() {
                            None => Some(Fullscreen::Borderless(None)),
                            Some(_) => None
                        })
                    }),
                    Some(VirtualKeyCode::F5) => {
                        log::info!("Reseting Game...");
                        ctx = app.suspend();
                        app = Application::<InfinityLoop, GlutinContext>::new(None).unwrap();
                        app.resume(||Ok(ctx.take().unwrap()));
                    },
                    Some(VirtualKeyCode::Return) => {
                        app.on_press(pos.x as f32, pos.y as f32, 1 + touch_stack.len() as u64);
                        touch_stack.push_back(pos);
                    },
                    Some(VirtualKeyCode::Back) => {
                        if let Some(pos) = touch_stack.pop_back() {
                            app.on_release(pos.x as f32, pos.y as f32, 1 + touch_stack.len() as u64);
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
                if app.should_save() {
                    app.save(|s| Ok(std::fs::write(save_file, s)?)).unwrap();
                }
                app.process_timeouts();
            },
            Event::LoopDestroyed => {
                app.suspend();
                app.save(|s| Ok(std::fs::write(save_file, s)?)).unwrap();
            },
            _ => {}
        }
    });

}
