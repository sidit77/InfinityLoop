use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use css_color_parser::Color;
use std::time::Duration;
use miniserde::json;
use crate::game::{Game, GameStyle, GameEvent};
use crate::world::WorldSave;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[allow(unused_macros)]
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&format_args!($($t)*).to_string().into()))
}

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

pub fn vibrate(duration: Duration){
    web_sys::window().unwrap().navigator().vibrate_with_duration(duration.as_millis() as u32);
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
        canvas.get_context("webgl2").unwrap().unwrap().dyn_into::<web_sys::WebGl2RenderingContext>()?,
        GameStyle {
            foreground: css.get_property_value("color")?.parse::<Color>().unwrap(),
            background: css.get_property_value("background-color")?.parse::<Color>().unwrap()
        }
    )?));

    {
        let game = game.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let canvas = event
                .current_target()
                .unwrap()
                .dyn_into::<web_sys::HtmlCanvasElement>()
                .unwrap();
            game.borrow_mut().on_event(GameEvent::Click(
                event.client_x() as f32 / canvas.client_width() as f32,
                event.client_y() as f32 / canvas.client_height() as f32
            ));
            event.prevent_default();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let game = game.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::CustomEvent| {
            let detail = event.detail().as_string().unwrap();
            game.borrow_mut().on_event(GameEvent::SaveReceived(detail.as_str()));
        }) as Box<dyn FnMut(_)>);
        window.add_event_listener_with_callback("save-received", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let game = game.clone();
        let closure = Closure::wrap(Box::new(move || {
            game.borrow_mut().on_event(GameEvent::SolveButton);
        }) as Box<dyn FnMut()>);
        get_element::<web_sys::HtmlElement>("new-button")
            .set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    {
        let game = game.clone();
        let closure = Closure::wrap(Box::new(move || {
            game.borrow_mut().on_event(GameEvent::ScrambleButton);
        }) as Box<dyn FnMut()>);
        get_element::<web_sys::HtmlElement>("scramble-button")
            .set_onclick(Some(closure.as_ref().unchecked_ref()));
        closure.forget();
    }

    {
        let game = game.clone();
        let closure = Closure::wrap(Box::new(move || {
            game.borrow_mut().on_event(GameEvent::Quitting);
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

                game.borrow_mut().on_event(GameEvent::Resize(width, height));
            }
        }
        let dt = performance.now() - time;
        time = performance.now();

        game.borrow_mut().render(perf_to_duration(dt));

        {
            let game = game.borrow_mut();
            let nl = Some((game.world().seed(), game.finished()));
            if nl != level {
                level = nl;
                let (level, finished) = level.unwrap();
                level_span.set_inner_text(format!("#{}", level).as_str());
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

fn perf_to_duration(amt: f64) -> Duration {
    let secs = (amt as u64) / 1_000;
    let nanos = ((amt as u32) % 1_000) * 1_000_000;
    Duration::new(secs, nanos)
}

struct SaveManager {
    storage: web_sys::Storage,
    save: Option<WorldSave>
}

impl SaveManager {
    const STORAGE_KEY: &'static str = "current-level";

    pub fn new() -> Self {
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        let save: Option<String> = storage.get_item(Self::STORAGE_KEY).unwrap();
        let save = match save {
            None => None,
            Some(save) => match json::from_str(save.as_str()) {
                Ok(save) => Some(save),
                Err(e) => {
                    console_log!("{}", e);
                    None
                }
            }
        };
        Self {
            storage,
            save
        }
    }

    pub fn load_world(&mut self) -> Option<WorldSave> {
        self.save.clone()
    }

    pub fn save_world(&mut self, world: &WorldSave) {
        let flush = world.seed > self.save.as_ref().map(|ws| ws.seed).unwrap_or(u64::MIN);
        self.save = Some(world.clone());
        if flush {
            self.flush();
        }
    }

    pub fn handle_world_update(&mut self, world_json: &str) -> Option<WorldSave> {
        match json::from_str::<WorldSave>(world_json) {
            Ok(remote_save) => match self.load_world() {
                Some(local_save) => if remote_save.seed > local_save.seed {
                    self.save_world(&remote_save);
                    Some(remote_save)
                } else {
                    None
                },
                None => {
                    self.save_world(&remote_save);
                    Some(remote_save)
                }
            },
            Err(e) => {
                console_log!("{}", e);
                None
            }
        }
    }

    pub fn flush(&mut self) {
        if let Some(world) = &self.save {
            let world_json = json::to_string(world);
            self.storage.set_item(Self::STORAGE_KEY, world_json.as_str()).expect("can't save");

            web_sys::window().unwrap().dispatch_event(
                &web_sys::CustomEvent::new_with_event_init_dict("saved",
                      &web_sys::CustomEventInit::new().detail(&world_json.into())
                ).unwrap()
            ).unwrap();
        }

    }

}