use std::panic;
use log::{debug, Level};
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(start)]
pub fn main_js() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Debug).expect("error initializing logger");

    debug!("Hello World!");
}