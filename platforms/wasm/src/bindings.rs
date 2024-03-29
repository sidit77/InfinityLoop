use serde::Deserialize;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::*;



#[wasm_bindgen(module = "/src/bindings.js")]
extern "C" {
    fn set_callback(callback: JsValue);
    pub fn request_redraw();
    pub fn set_timeout(millis: i32) -> i32;
    pub fn clear_timeout(handle: i32);
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub enum JsEvent {
    Resize {
        width: u32,
        height: u32
    },
    MouseMove {
        x: i32,
        y: i32
    },
    MouseDown,
    MouseUp,
    MouseWheel {
        amt: f32
    },
    Touch {
        phase: TouchPhase,
        x: i32,
        y: i32,
        id: u32
    },
    Redraw,
    Unloading,
    Timeout
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Deserialize)]
pub enum TouchPhase {
    #[serde(rename = "touchstart")]
    Start,
    #[serde(rename = "touchmove")]
    Move,
    #[serde(rename = "touchend")]
    End,
    #[serde(rename = "touchcancel")]
    Cancel
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
