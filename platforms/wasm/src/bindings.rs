use serde::Deserialize;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;



#[wasm_bindgen(module = "/src/bindings.js")]
extern "C" {
    fn set_callback(callback: JsValue);
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub enum JsEvent {
    MouseMove {
        x: i32,
        y: i32
    },
    MouseDown,
    MouseUp,
    MouseWheel {
        amt: f32
    }
}

pub fn set_js_callback<F: FnMut(JsEvent) + 'static>(mut f: F) {
    let closure: Closure<dyn FnMut(JsValue)> = Closure::new(move |value| {
        match serde_wasm_bindgen::from_value(value) {
            Ok(event) => f(event),
            Err(err) => log::warn!("Event error: {}", err)
        }
    });
    set_callback(closure.into_js_value());
}
