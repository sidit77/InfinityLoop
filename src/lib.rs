use std::f64;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::rc::Rc;

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

fn canvas() -> web_sys::HtmlCanvasElement {
    document()
        .get_element_by_id("canvas")
        .expect("should have a element called 'canvas'")
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .expect("'canvas' should be a canvas")
}

fn context(canvas: &web_sys::HtmlCanvasElement) -> web_sys::CanvasRenderingContext2d {
    canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap()
}

#[wasm_bindgen(start)]
pub fn start() {

    let window = window();
    let canvas = canvas();
    let context = context(&canvas);

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut i = 0.0;
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        canvas.set_width(window.inner_width().unwrap().as_f64().unwrap() as u32 - 4);
        canvas.set_height(window.inner_height().unwrap().as_f64().unwrap() as u32 - 4);
        context.clear_rect(0.0, 0.0, canvas.width() as f64, canvas.height() as f64);
        i += 0.1;

        context.begin_path();

        // Draw the outer circle.
        //context.move_to(i, i);
        context
            .arc(75.0 + i, 75.0 + i, 50.0, 0.0, f64::consts::PI * 2.0)
            .unwrap();

        // Draw the mouth.
        context.move_to(110.0 + i, 75.0 + i);
        context.arc(75.0 + i, 75.0 + i, 35.0, 0.0, f64::consts::PI).unwrap();

        // Draw the left eye.
        context.move_to(65.0 + i, 65.0 + i);
        context
            .arc(60.0 + i, 65.0 + i, 5.0, 0.0, f64::consts::PI * 2.0)
            .unwrap();

        // Draw the right eye.
        context.move_to(95.0 + i, 65.0 + i);
        context
            .arc(90.0 + i, 65.0 + i, 5.0, 0.0, f64::consts::PI * 2.0)
            .unwrap();

        context.stroke();

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

}