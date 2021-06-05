use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[allow(unused_macros)]
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&format_args!($($t)*).to_string().into()))
}

use crate::game::Game;

mod shader;
mod camera;
mod game;
mod meshes;
mod intersection;
mod world;

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window().unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug)]
    console_error_panic_hook::set_once();

    let window = web_sys::window().unwrap();
    let canvas = window.document().unwrap().get_element_by_id("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>()?;
    let performance = window.performance().unwrap();

    let game = Rc::new(RefCell::new(Game::new(
        canvas.get_context("webgl2").unwrap().unwrap().dyn_into::<WebGl2RenderingContext>()?
    )?));

    {
        let game = game.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let canvas = event
                .current_target()
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();
            game.borrow_mut().mouse_down(
                event.client_x() as f32 / canvas.client_width() as f32,
                event.client_y() as f32 / canvas.client_height() as f32
            );
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut time = performance.now();
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        {
            let dpi = window.device_pixel_ratio() as f32;
            let width = (canvas.client_width() as f32 * dpi) as u32;
            let height = (canvas.client_height() as f32 * dpi) as u32 ;

            if width != canvas.width() || height != canvas.height(){
                canvas.set_width(width);
                canvas.set_height(height);


                game.borrow_mut().resize(width, height);
            }
        }
        let dt = performance.now() - time;
        time = performance.now();

        game.borrow_mut().render(dt);

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}
