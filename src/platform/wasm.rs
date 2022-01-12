use glow::Context;
use log::Level;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use winit::window::{Window, WindowBuilder};
use winit::dpi::PhysicalSize;
use winit::event_loop::EventLoopWindowTarget;
use winit::platform::web::WindowBuilderExtWebSys;
use crate::opengl::Context;

#[wasm_bindgen(start)]
pub fn run() {
    crate::main();
}

pub fn setup_logger(level: Level) {
    console_log::init_with_level(level).expect("error initializing logger");
}

pub trait WindowBuilderExt {
    fn build_context<T>(self, el: &EventLoopWindowTarget<T>) -> (WasmWindow, Context);
}

impl WindowBuilderExt for WindowBuilder {

    fn build_context<T>(self, el: &EventLoopWindowTarget<T>) -> (WasmWindow, Context) {
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

        let window = self.with_canvas(Some(canvas)).build(&el).unwrap();
        let gl = Context::from_webgl2_context(webgl2_context);

        (WasmWindow::new(window), gl)
    }

}

pub struct WasmWindow(Window);

impl WasmWindow {

    pub fn new(window: Window) -> Self {
        Self::adjust_size(&window);
        WasmWindow(window)
    }

    pub fn window(&self) -> &Window {
        &self.0
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

    pub fn swap_buffers(&self) -> Result<(), ()> {
        Self::adjust_size(&self.0);
        Ok(())
    }

    pub fn resize(&self, _size: PhysicalSize<u32>) {

    }

}