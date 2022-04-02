use std::ops::Deref;
use std::panic;
use log::Level;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use winit::dpi::{PhysicalPosition, PhysicalSize};
use winit::event::{ElementState, Event, MouseButton, MouseScrollDelta, Touch, TouchPhase, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use winit::platform::web::WindowBuilderExtWebSys;
use winit::window::{Window, WindowBuilder};
use infinity_loop::export::{AppContext, Application, Context, GlowContext, Result};
use infinity_loop::InfinityLoop;

struct WasmContext(Window, Context);

impl WasmContext {
    fn new(el: &EventLoopWindowTarget<()>) -> Result<Self> {
        let canvas = web_sys::window().unwrap()
            .document().unwrap()
            .get_element_by_id("canvas").unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        let webgl2_context = canvas
            .get_context("webgl2").unwrap().unwrap()
            .dyn_into::<web_sys::WebGl2RenderingContext>().unwrap();

        let window = WindowBuilder::new()
            .with_canvas(Some(canvas))
            .build(el).unwrap();
        let gl = GlowContext::from_webgl2_context(webgl2_context);
        Ok(Self(window, Context::from_glow(gl)))
    }
}

impl Deref for WasmContext {
    type Target = Context;
    fn deref(&self) -> &Self::Target {
        self.gl()
    }
}

impl AppContext for WasmContext {
    fn gl(&self) -> &Context {
        &self.1
    }

    fn screen_size(&self) -> (u32, u32) {
        let size = self.0.inner_size();
        (size.width, size.height)
    }
}

#[wasm_bindgen(start)]
pub fn main_js() -> std::result::Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Debug).expect("error initializing logger");

    let mut app = Application::<InfinityLoop, WasmContext>::new().unwrap();

    let event_loop = EventLoop::new();

    app.resume(|| WasmContext::new(&event_loop));

    let mut pos = PhysicalPosition::new(0.0, 0.0);
    let mut down = false;

    event_loop.run(move |event, _event_loop, control_flow| {
        *control_flow = match app.should_redraw() {
            true => ControlFlow::Poll,
            false => ControlFlow::Wait
        };
        match event {
            Event::WindowEvent { event, ..} => match event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(size) => {
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
                app.with_ctx(|ctx| {
                    let window = web_sys::window().unwrap();
                    let width = window.inner_width().unwrap().as_f64().unwrap() as u32;
                    let height = window.inner_height().unwrap().as_f64().unwrap() as u32;
                    let size = ctx.0.inner_size();
                    if size.width != width || size.height != height {
                        ctx.0.set_inner_size(PhysicalSize::new(width, height));
                    }
                })
            },
            Event::MainEventsCleared =>  {
                if app.should_redraw() {
                    app.with_ctx(|ctx| ctx.0.request_redraw());
                }
            },
            Event::LoopDestroyed => {
                app.suspend();
            },
            _ => {}
        }
    })
}