mod bindings;

use std::ops::{Deref};
use std::panic;
use log::Level;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement,WebGl2RenderingContext};
use infinity_loop::export::{AppContext, Application, Context, GlowContext, Result};
use infinity_loop::InfinityLoop;

use crate::bindings::{JsEvent, request_redraw, set_js_callback, TouchPhase};

/*
fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window().unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}
*/

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

    /*
    fn client_to_screen(&self, x: i32, y: i32) -> (f32, f32) {
        //let rect = self
        //    .get_bounding_client_rect();
        //log::debug!("x: {} y: {}",
        //    (event.client_x() as f32), // - rect.x() as f32
        //    (event.client_y() as f32)  // - rect.y() as f32
        //);
        let dpi = web_sys::window().unwrap().device_pixel_ratio() as f32;
        (x as f32 * dpi, y as f32 * dpi)
    }
*/
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
    set_js_callback(move |event| {
        match event {
            JsEvent::Redraw => app.redraw(),
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
            JsEvent::Unloading => app.save(|s| Ok(storage.set_item(&save_key, &s).unwrap())).unwrap()
        }
        if app.should_save() {
            app.save(|s| Ok(storage.set_item(save_key, &s).unwrap())).unwrap();
        }
        if app.should_redraw() {
            request_redraw();
        }
    });
    request_redraw();
    /*
    let save_key = "savestate";

    let window = web_sys::window().unwrap();
    let storage = window.local_storage().unwrap().unwrap();
    let canvas = window.document().unwrap()
        .get_element_by_id("canvas").unwrap()
        .dyn_into::<HtmlCanvasElement>().unwrap();

    let app = Rc::new(RefCell::new(Application::<InfinityLoop, WasmContext>::new(storage.get_item(save_key).unwrap()).unwrap()));

    app.as_ref().borrow_mut().resume(|| WasmContext::new(&canvas));

    let input = Rc::new(RefCell::new(InputState::default()));
    let f = Rc::new(RefCell::new(None));

    let after_events = handle_events(app.clone(), f.clone());

    register_resize(app.clone(), after_events.clone())?;
    register_beforeunload(app.clone(), save_key)?;
    register_mousemove(&canvas, app.clone(), input.clone(), after_events.clone())?;
    register_mousedown(&canvas, app.clone(), input.clone(),after_events.clone())?;
    register_mouseup(&canvas, app.clone(), input.clone(),after_events.clone())?;
    register_wheel(&canvas, app.clone(), input.clone(),after_events.clone())?;
    register_touchstart(&canvas, app.clone(), after_events.clone())?;
    register_touchmove(&canvas, app.clone(), after_events.clone())?;
    register_touchend(&canvas, app.clone(), after_events.clone())?;

    let g = f.clone();
    *g.as_ref().borrow_mut() = Some(Closure::new(move || {
        {
            let mut app = app.as_ref().borrow_mut();

            if app.should_save() {
                app.save(|s| Ok(storage.set_item(save_key, &s).unwrap())).unwrap();
            }

            app.redraw();
        }
        after_events.as_ref().borrow_mut()();

    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
    */
    Ok(())

}

/*
type RcApp = Rc<RefCell<Application<InfinityLoop, WasmContext>>>;
type RcInput = Rc<RefCell<InputState>>;
type RcClosure = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;
type RcFn = Rc<RefCell<dyn FnMut()>>;

fn handle_events(app: RcApp, anim: RcClosure) -> RcFn {
    let window = web_sys::window().unwrap();
    let callback: Closure<dyn FnMut()> = Closure::new({
        let anim = anim.clone();
        move || {
            request_animation_frame(anim.borrow().as_ref().unwrap());
        }
    });
    let mut last_timeout = None;
    let mut last_timeout_code = 0;
    Rc::new(RefCell::new(move || {
        let mut app = app.as_ref().borrow_mut();
        app.process_timeouts();
        if app.should_redraw() {
            request_animation_frame(anim.borrow().as_ref().unwrap());
        }
        let next_timeout = app.next_timeout();
        if next_timeout != last_timeout {
            if last_timeout.is_some() {
                window.clear_timeout_with_handle(last_timeout_code);
            }
            if let Some(timeout) = next_timeout {
                let millis = timeout.saturating_duration_since(Instant::now()).as_millis() as i32;
                last_timeout_code = window.set_timeout_with_callback_and_timeout_and_arguments_0(callback.as_ref().unchecked_ref(), millis).unwrap();
            }
            last_timeout = next_timeout;
        }
    }))
}

fn register_resize(app: RcApp, after_events: RcFn) -> std::result::Result<(), JsValue> {
    let closure = Closure::<dyn Fn(_)>::new(move |_: Event| {
        {
            let mut app = app.as_ref().borrow_mut();
            app.with_ctx(WasmContext::resize);
            let size = app.with_ctx(WasmContext::screen_size);
            app.set_screen_size(size);
        }
        after_events.as_ref().borrow_mut()();
    });
    web_sys::window().unwrap().add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn register_beforeunload(app: RcApp, save_key: &str) -> std::result::Result<(), JsValue> {
    let save_key = save_key.to_string();
    let closure = Closure::<dyn Fn(_)>::new(move |_: Event| {
        let mut app = app.as_ref().borrow_mut();
        let storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();
        app.save(|s| Ok(storage.set_item(&save_key, &s).unwrap())).unwrap();
    });
    web_sys::window().unwrap().add_event_listener_with_callback("beforeunload", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn register_mousemove(canvas: &HtmlCanvasElement, app: RcApp, input: RcInput, after_events: RcFn) -> std::result::Result<(), JsValue> {
    let closure = Closure::<dyn Fn(_)>::new(move |event: MouseEvent| {
        {
            let mut input = input.as_ref().borrow_mut();
            let mut app = app.as_ref().borrow_mut();
            let (x,y) = app.with_ctx(|ctx|ctx.client_to_screen(event.client_x(), event.client_y()));
            input.mouse_x = x;
            input.mouse_y = y;
            if input.mouse_down {
                app.on_move(x, y, 0);
            }
            event.prevent_default();
        }
        after_events.as_ref().borrow_mut()();
    });
    canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn register_mousedown(canvas: &HtmlCanvasElement, app: RcApp, input: RcInput, after_events: RcFn) -> std::result::Result<(), JsValue> {
    let closure = Closure::<dyn Fn(_)>::new(move |event: MouseEvent| {
        {
            let mut input = input.as_ref().borrow_mut();
            let mut app = app.as_ref().borrow_mut();
            input.mouse_down = true;
            app.on_press(input.mouse_x, input.mouse_y, 0);
            event.prevent_default();
        }
        after_events.as_ref().borrow_mut()();
    });
    canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn register_mouseup(canvas: &HtmlCanvasElement, app: RcApp, input: RcInput, after_events: RcFn) -> std::result::Result<(), JsValue> {
    let closure = Closure::<dyn Fn(_)>::new(move |event: MouseEvent| {
        {
            let mut input = input.as_ref().borrow_mut();
            let mut app = app.as_ref().borrow_mut();
            input.mouse_down = false;
            app.on_release(input.mouse_x, input.mouse_y, 0);
            event.prevent_default();
        }
        after_events.as_ref().borrow_mut()();
    });
    canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}


fn register_wheel(canvas: &HtmlCanvasElement, app: RcApp, input: RcInput, after_events: RcFn) -> std::result::Result<(), JsValue> {
    let closure = Closure::<dyn Fn(_)>::new(move |event: WheelEvent| {
        {
            let input = input.as_ref().borrow();
            let mut app = app.as_ref().borrow_mut();
            let dy = match event.delta_mode() {
                WheelEvent::DOM_DELTA_PIXEL => -event.delta_y() as f32 / 100.0,
                WheelEvent::DOM_DELTA_LINE => -event.delta_y() as f32 / 2.0,
                _ => 0.0
            };
            app.on_mouse_wheel(input.mouse_x, input.mouse_y, dy);
            event.prevent_default();
        }
        after_events.as_ref().borrow_mut()();
    });
    canvas.add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn register_touchstart(canvas: &HtmlCanvasElement, app: RcApp, after_events: RcFn) -> std::result::Result<(), JsValue> {
    let closure = Closure::<dyn Fn(_)>::new(move |event: TouchEvent| {
        {
            let mut app = app.as_ref().borrow_mut();
            let changed = event.changed_touches();
            for i in 0..changed.length() {
                if let Some(touch) = changed.item(i) {
                    let (x,y) = app.with_ctx(|ctx|ctx.client_to_screen(touch.client_x(), touch.client_y()));
                    app.on_press(x, y, touch.identifier() as u64);
                }
            }
            event.prevent_default();
        }
        after_events.as_ref().borrow_mut()();
    });
    canvas.add_event_listener_with_callback("touchstart", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn register_touchmove(canvas: &HtmlCanvasElement, app: RcApp, after_events: RcFn) -> std::result::Result<(), JsValue> {
    let closure = Closure::<dyn Fn(_)>::new(move |event: TouchEvent| {
        {
            let mut app = app.as_ref().borrow_mut();
            let changed = event.changed_touches();
            for i in 0..changed.length() {
                if let Some(touch) = changed.item(i) {
                    let (x,y) = app.with_ctx(|ctx|ctx.client_to_screen(touch.client_x(), touch.client_y()));
                    app.on_move(x, y, touch.identifier() as u64);
                }
            }
            event.prevent_default();
        }
        after_events.as_ref().borrow_mut()();
    });
    canvas.add_event_listener_with_callback("touchmove", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn register_touchend(canvas: &HtmlCanvasElement, app: RcApp, after_events: RcFn) -> std::result::Result<(), JsValue> {
    let closure = Closure::<dyn Fn(_)>::new(move |event: TouchEvent| {
        {
            let mut app = app.as_ref().borrow_mut();
            let changed = event.changed_touches();
            for i in 0..changed.length() {
                if let Some(touch) = changed.item(i) {
                    let (x,y) = app.with_ctx(|ctx|ctx.client_to_screen(touch.client_x(), touch.client_y()));
                    app.on_release(x, y, touch.identifier() as u64);
                }
            }
            event.prevent_default();
        }
        after_events.as_ref().borrow_mut()();
    });
    canvas.add_event_listener_with_callback("touchend", closure.as_ref().unchecked_ref())?;
    canvas.add_event_listener_with_callback("touchcancel", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}
*/
//{
//    let app = app.clone();
//    let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
//        let rect = event
//            .current_target()
//            .unwrap()
//            .dyn_into::<web_sys::HtmlCanvasElement>()
//            .unwrap()
//            .get_bounding_client_rect();
//        app.borrow_mut().on_press(
//            (event.client_x() as f32 - rect.x() as f32) / rect.width()   as f32,
//            (event.client_y() as f32 - rect.y() as f32) / rect.height()  as f32,
//            0
//        );
//        event.prevent_default();
//    }) as Box<dyn FnMut(_)>);
//    canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
//    closure.forget();
//}

//let mut pos = PhysicalPosition::new(0.0, 0.0);
//let mut down = false;
//
//event_loop.run(move |event, _event_loop, control_flow| {
//    *control_flow = match app.should_redraw() {
//        true => ControlFlow::Poll,
//        false => ControlFlow::Wait
//    };
//    match event {
//        Event::WindowEvent { event, ..} => match event {
//            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
//            WindowEvent::Resized(size) => {
//                app.set_screen_size(size.into())
//            },
//            WindowEvent::CursorMoved { position,.. } => {
//                pos = position;
//                if down {
//                    app.on_move(pos.x as f32, pos.y as f32, 0);
//                }
//            },
//            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, ..}  => {
//                app.on_press(pos.x as f32, pos.y as f32, 0);
//                down = true;
//            },
//            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Left, ..} => {
//                app.on_release(pos.x as f32, pos.y as f32, 0);
//                down = false;
//            },
//            WindowEvent::MouseWheel { delta, .. } => {
//                let dy = match delta {
//                    MouseScrollDelta::LineDelta(_, dy) => dy,
//                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0
//                };
//                app.on_mouse_wheel(pos.x as f32, pos.y as f32, dy)
//            },
//            WindowEvent::Touch(Touch{ phase, location, id, .. }) => match phase {
//                TouchPhase::Started => app.on_press(location.x as f32, location.y as f32, id),
//                TouchPhase::Moved => app.on_move(location.x as f32, location.y as f32, id),
//                TouchPhase::Ended => app.on_release(location.x as f32, location.y as f32, id),
//                TouchPhase::Cancelled => app.on_release(location.x as f32, location.y as f32, id)
//            },
//            _ => {}
//        },
//        Event::RedrawRequested(_) => {
//            app.redraw();
//            app.with_ctx(|ctx| {
//                let window = web_sys::window().unwrap();
//                let width = window.inner_width().unwrap().as_f64().unwrap() as u32;
//                let height = window.inner_height().unwrap().as_f64().unwrap() as u32;
//                let size = ctx.0.inner_size();
//                if size.width != width || size.height != height {
//                    ctx.0.set_inner_size(PhysicalSize::new(width, height));
//                }
//            })
//        },
//        Event::MainEventsCleared =>  {
//            if app.should_redraw() {
//                app.with_ctx(|ctx| ctx.0.request_redraw());
//            }
//        },
//        Event::LoopDestroyed => {
//            app.suspend();
//        },
//        _ => {}
//    }
//})