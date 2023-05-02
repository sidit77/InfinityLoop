mod bindings;

use std::ops::{Deref};
use std::panic;
use instant::Instant;
use log::Level;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement,WebGl2RenderingContext};
use infinity_loop::export::{AppContext, Application, Context, GlowContext, Result};
use infinity_loop::InfinityLoop;

use crate::bindings::{clear_timeout, JsEvent, request_redraw, set_js_callback, set_timeout, TouchPhase};


struct WasmContext(HtmlCanvasElement, Context);

impl WasmContext {
    fn new(canvas: &HtmlCanvasElement) -> Result<Self> {
        let webgl2_context = canvas
            .get_context("webgl2").unwrap().unwrap()
            .dyn_into::<WebGl2RenderingContext>().unwrap();

        let gl = GlowContext::from_webgl2_context(webgl2_context);
        let result = Self(canvas.clone(), Context::from_glow(gl));
        result.resize();
        Ok(result)
    }

    fn resize(&self) {
        let dpi = web_sys::window().unwrap().device_pixel_ratio() as f32;
        let width = (self.0.client_width() as f32 * dpi) as u32;
        let height = (self.0.client_height() as f32 * dpi) as u32 ;

        self.0.set_width(width);
        self.0.set_height(height);
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
        (self.0.width(), self.0.height())
    }
}

#[derive(Default, Copy, Clone)]
struct InputState {
    mouse_x: f32,
    mouse_y: f32,
    mouse_down: bool
}


fn main() -> std::result::Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(Level::Debug).expect("error initializing logger");

    let save_key = "savestate";

    let window = web_sys::window().unwrap();
    let storage = window.local_storage().unwrap().unwrap();
    let canvas = window.document().unwrap()
        .get_element_by_id("canvas").unwrap()
        .dyn_into::<HtmlCanvasElement>().unwrap();
    let mut app = Application::<InfinityLoop, WasmContext>::new(storage.get_item(save_key).unwrap()).unwrap();
    app.resume(|| WasmContext::new(&canvas));

    let mut input = InputState::default();
    let mut redraw_queued = false;
    let mut current_timeout: Option<(i32, Instant)> = None;
    set_js_callback(move |event| {
        match event {
            JsEvent::Redraw => {
                app.redraw();
                redraw_queued = false;
            },
            JsEvent::Resize { width, height } => app.set_screen_size((width, height)),
            JsEvent::MouseMove { x, y } => {
                input.mouse_x = x as f32;
                input.mouse_y = y as f32;
                if input.mouse_down {
                    app.on_move(x as f32, y as f32, 0);
                }
            }
            JsEvent::MouseDown => {
                input.mouse_down = true;
                app.on_press(input.mouse_x, input.mouse_y, 0);
            }
            JsEvent::MouseUp => {
                input.mouse_down = false;
                app.on_release(input.mouse_x, input.mouse_y, 0);
            }
            JsEvent::MouseWheel { amt } => app.on_mouse_wheel(input.mouse_x, input.mouse_y, amt),
            JsEvent::Touch { phase, x, y, id } => match phase {
                TouchPhase::Start => app.on_press(x as f32, y as f32, id as u64),
                TouchPhase::Move => app.on_move(x as f32, y as f32, id as u64),
                TouchPhase::End | TouchPhase::Cancel => app.on_release(x as f32, y as f32, id as u64),
            }
            JsEvent::Unloading => app.save(|s| Ok(storage.set_item(&save_key, &s).unwrap())).unwrap(),
            JsEvent::Timeout => {
                app.process_timeouts();
                current_timeout = None;
            }
        }
        if app.should_save() {
            app.save(|s| Ok(storage.set_item(save_key, &s).unwrap())).unwrap();
        }
        if app.should_redraw() && !redraw_queued {
            request_redraw();
            redraw_queued = true;
        }
        let next_timeout = app.next_timeout();
        if current_timeout.map(|(_, i)| i) != next_timeout {
            if let Some((id, _)) = current_timeout.take() {
                clear_timeout(id);
            }
            current_timeout = next_timeout.map(|end|{
                let millis = end.saturating_duration_since(Instant::now()).as_millis() as i32;
                (set_timeout(millis), end)
            });
        }
    });
    request_redraw();
    Ok(())

}
