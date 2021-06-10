use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext;
use css_color_parser::Color;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[allow(unused_macros)]
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&format_args!($($t)*).to_string().into()))
}

use crate::game::{Game, GameStyle};
use crate::world::WorldSave;

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

fn get_element<T: JsCast>(id: &str) -> T {
    web_sys::window().expect("can't find window")
        .document().expect("can't find document")
        .get_element_by_id(id).expect(format!("can't find element {}", id).as_str())
        .dyn_into::<T>().expect("can't convert the the desired type")
}

fn load_world() -> Option<WorldSave> {
    let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    let save: Option<String> = storage.get_item("current-level").unwrap();
    match save {
        None => None,
        Some(save) => match save.parse::<WorldSave>() {
            Ok(save) => Some(save),
            Err(e) => {
                console_log!("{}", e);
                None
            }
        }
    }
}

fn save_world(world: &WorldSave) {
    let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
    storage.set_item("current-level", world.to_string().as_str()).expect("can't save");
}

#[wasm_bindgen(start)]
pub fn main_js() -> Result<(), JsValue> {
    #[cfg(debug)]
    console_error_panic_hook::set_once();

    let window = web_sys::window().unwrap();
    let canvas = get_element::<web_sys::HtmlCanvasElement>("canvas");
    let performance = window.performance().unwrap();
    let css = window.get_computed_style(&canvas)?.unwrap();
    let level_span = get_element::<web_sys::HtmlSpanElement>("current-level");

    let game = Rc::new(RefCell::new(Game::new(
        canvas.get_context("webgl2").unwrap().unwrap().dyn_into::<WebGl2RenderingContext>()?,
        GameStyle {
            foreground: css.get_property_value("color")?.parse::<Color>().unwrap(),
            background: css.get_property_value("background-color")?.parse::<Color>().unwrap()
        },
        load_world()
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
            event.prevent_default();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let game = game.clone();
        let closure = Closure::wrap(Box::new(move || {
            game.borrow_mut().new_level();
        }) as Box<dyn FnMut()>);
        get_element::<web_sys::HtmlElement>("new-button")
            .set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    {
        let game = game.clone();
        let closure = Closure::wrap(Box::new(move || {
            game.borrow_mut().scramble_level();
        }) as Box<dyn FnMut()>);
        get_element::<web_sys::HtmlElement>("scramble-button")
            .set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    {
        let game = game.clone();
        let closure = Closure::wrap(Box::new(move || {
            save_world(&WorldSave::from(game.borrow_mut().world()));
            console_log!("Saved");
        }) as Box<dyn FnMut()>);
        window.set_onbeforeunload(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut time = performance.now();
    let mut level = None;
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

        {
            let game = game.borrow_mut();
            let nl = Some((game.world().seed(), game.finished()));
            if nl != level {
                level = nl;
                let (level, finished) = level.unwrap();
                level_span.set_inner_text(format!("#{}", level).as_str());
                save_world(&WorldSave::from(game.world()));
                level_span.style().set_css_text(if finished {
                    "filter: invert(100%);"
                } else {
                    ""
                });
            }
        }

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}
