use std::panic;
use log::Level;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoopWindowTarget;
use winit::platform::web::WindowBuilderExtWebSys;
use winit::window::{Window, WindowBuilder};
use infinity_loop::{Game, GlowContext, InfinityLoop, Platform, PlatformWindow};

struct WasmWindow(Window);

impl WasmWindow {

    fn new(window: Window) -> Self {
        Self::adjust_size(&window);
        WasmWindow(window)
    }

    fn adjust_size(w: &Window) {
        let window = web_sys::window().unwrap();
        let width = window.inner_width().unwrap().as_f64().unwrap() as u32;
        let height = window.inner_height().unwrap().as_f64().unwrap() as u32;
        let size = w.inner_size();
        if size.width != width || size.height != height {
            w.set_inner_size(PhysicalSize::new(width, height));
        }
    }

}

impl PlatformWindow for WasmWindow {
    fn window(&self) -> &Window {
        &self.0
    }

    fn swap_buffers(&self) {
        Self::adjust_size(&self.0);
    }

    fn resize_surface(&self, _: PhysicalSize<u32>) { }
}

struct WasmPlatform;

impl Platform for WasmPlatform {
    type Window = WasmWindow;

    fn create_context<T>(el: &EventLoopWindowTarget<T>) -> (Self::Window, GlowContext) {
        let canvas = web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .get_element_by_id("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        let webgl2_context = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<web_sys::WebGl2RenderingContext>()
            .unwrap();

        let window = WindowBuilder::new()
            .with_canvas(Some(canvas))
            .build(el).unwrap();
        let gl = GlowContext::from_webgl2_context(webgl2_context);

        (WasmWindow::new(window), gl)
    }
}


#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Debug).expect("error initializing logger");

    InfinityLoop::run::<WasmPlatform>()
}