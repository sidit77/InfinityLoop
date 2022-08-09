use std::cell::RefCell;
use std::ops::{Deref};
use std::panic;
use std::rc::Rc;
use log::Level;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, MouseEvent, WebGl2RenderingContext, WheelEvent};
use infinity_loop::export::{AppContext, Application, Context, GlowContext, Result};
use infinity_loop::InfinityLoop;

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window().unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

struct WasmContext(HtmlCanvasElement, Context);

impl WasmContext {
    fn new(canvas: &HtmlCanvasElement) -> Result<Self> {
        let webgl2_context = canvas
            .get_context("webgl2").unwrap().unwrap()
            .dyn_into::<WebGl2RenderingContext>().unwrap();

        let gl = GlowContext::from_webgl2_context(webgl2_context);
        Ok(Self(canvas.clone(), Context::from_glow(gl)))
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

    let window = web_sys::window().unwrap();
    let canvas = window.document().unwrap()
        .get_element_by_id("canvas").unwrap()
        .dyn_into::<HtmlCanvasElement>().unwrap();

    let app = Rc::new(RefCell::new(Application::<InfinityLoop, WasmContext>::new().unwrap()));

    app.as_ref().borrow_mut().resume(|| WasmContext::new(&canvas));

    let input = Rc::new(RefCell::new(InputState::default()));
    let f = Rc::new(RefCell::new(None));

    register_mousemove(&canvas, app.clone(), input.clone(), f.clone())?;
    register_mousedown(&canvas, app.clone(), input.clone(),f.clone())?;
    register_mouseup(&canvas, app.clone(), input.clone(),f.clone())?;
    register_wheel(&canvas, app.clone(), input.clone(),f.clone())?;

    let g = f.clone();
    *g.as_ref().borrow_mut() = Some(Closure::new(move || {
        {
            let dpi = window.device_pixel_ratio() as f32;
            let width = (canvas.client_width() as f32 * dpi) as u32;
            let height = (canvas.client_height() as f32 * dpi) as u32 ;

            if width != canvas.width() || height != canvas.height(){
                canvas.set_width(width);
                canvas.set_height(height);

                app.as_ref().borrow_mut().set_screen_size((width, height));
            }
        }

        app.as_ref().borrow_mut().redraw();

        if app.as_ref().borrow().should_redraw() {
            request_animation_frame(f.borrow().as_ref().unwrap());
        }

    }));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())

}

type RcApp = Rc<RefCell<Application<InfinityLoop, WasmContext>>>;
type RcInput = Rc<RefCell<InputState>>;
type RcClosure = Rc<RefCell<Option<Closure<dyn FnMut()>>>>;

fn register_mousemove(canvas: &HtmlCanvasElement, app: RcApp, input: RcInput, anim: RcClosure) -> std::result::Result<(), JsValue> {
    let closure = Closure::<dyn Fn(_)>::new(move |event: MouseEvent| {
        //let rect = event
        //    .current_target()
        //    .unwrap()
        //    .dyn_into::<HtmlCanvasElement>()
        //    .unwrap()
        //    .get_bounding_client_rect();
        //log::debug!("x: {} y: {}",
        //    (event.client_x() as f32), // - rect.x() as f32
        //    (event.client_y() as f32)  // - rect.y() as f32
        //);
        let mut input = input.as_ref().borrow_mut();
        input.mouse_x = event.client_x() as f32;
        input.mouse_y = event.client_y() as f32;
        if input.mouse_down {
            app.as_ref().borrow_mut().on_move(input.mouse_x, input.mouse_y, 0);
        }
        event.prevent_default();
        if app.as_ref().borrow().should_redraw() {
            request_animation_frame(anim.borrow().as_ref().unwrap());
        }
    });
    canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn register_mousedown(canvas: &HtmlCanvasElement, app: RcApp, input: RcInput, anim: RcClosure) -> std::result::Result<(), JsValue> {
    let closure = Closure::<dyn Fn(_)>::new(move |event: MouseEvent| {
        let mut input = input.as_ref().borrow_mut();
        input.mouse_down = true;
        app.as_ref().borrow_mut().on_press(input.mouse_x, input.mouse_y, 0);
        event.prevent_default();
        if app.as_ref().borrow().should_redraw() {
            request_animation_frame(anim.borrow().as_ref().unwrap());
        }
    });
    canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

fn register_mouseup(canvas: &HtmlCanvasElement, app: RcApp, input: RcInput, anim: RcClosure) -> std::result::Result<(), JsValue> {
    let closure = Closure::<dyn Fn(_)>::new(move |event: MouseEvent| {
        let mut input = input.as_ref().borrow_mut();
        input.mouse_down = false;
        app.as_ref().borrow_mut().on_release(input.mouse_x, input.mouse_y, 0);
        event.prevent_default();
        if app.as_ref().borrow().should_redraw() {
            request_animation_frame(anim.borrow().as_ref().unwrap());
        }
    });
    canvas.add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}


fn register_wheel(canvas: &HtmlCanvasElement, app: RcApp, input: RcInput, anim: RcClosure) -> std::result::Result<(), JsValue> {
    let closure = Closure::<dyn Fn(_)>::new(move |event: WheelEvent| {
        let input = input.as_ref().borrow();
        let dy = match event.delta_mode() {
            WheelEvent::DOM_DELTA_PIXEL => -event.delta_y() as f32 / 100.0,
            WheelEvent::DOM_DELTA_LINE => -event.delta_y() as f32,
            _ => 0.0
        };
        app.as_ref().borrow_mut().on_mouse_wheel(input.mouse_x, input.mouse_y, dy);
        event.prevent_default();
        if app.as_ref().borrow().should_redraw() {
            request_animation_frame(anim.borrow().as_ref().unwrap());
        }
    });
    canvas.add_event_listener_with_callback("wheel", closure.as_ref().unchecked_ref())?;
    closure.forget();
    Ok(())
}

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