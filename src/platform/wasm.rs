use log::Level;
use wasm_bindgen::prelude::*;
use winit::window::Window;
use winit::dpi::PhysicalSize;

#[wasm_bindgen(start)]
pub fn run() {
    crate::main();
}

pub fn setup_logger(level: Level) {
    console_log::init_with_level(level).expect("error initializing logger");
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