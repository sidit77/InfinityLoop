mod shader;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::WebGl2RenderingContext;
use crate::shader::compile_program;

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window().unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue>{

    let window = web_sys::window().unwrap();
    let canvas = window.document().unwrap().get_element_by_id("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>()?;
    let context = canvas.get_context("webgl2").unwrap().unwrap().dyn_into::<WebGl2RenderingContext>()?;

    let program = compile_program(&context, &[
        (WebGl2RenderingContext::VERTEX_SHADER, include_str!("shader/vertex.glsl")),
        (WebGl2RenderingContext::FRAGMENT_SHADER, include_str!("shader/fragment.glsl"))
    ])?;
    context.use_program(Some(&program));

    let vertices: [f32; 6] = [-0.7, -0.7, 0.7, -0.7, 0.0, 0.7];

    let buffer = context.create_buffer().ok_or("failed to create buffer")?;
    context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

    unsafe {
        let vert_array = js_sys::Float32Array::view(&vertices);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    context.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);
    context.enable_vertex_attrib_array(0);


    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut i = 0.0;
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        canvas.set_width(window.inner_width().unwrap().as_f64().unwrap() as u32 - 4);
        canvas.set_height(window.inner_height().unwrap().as_f64().unwrap() as u32 - 4);
        context.viewport(0, 0, canvas.width() as i32, canvas.height() as i32);
        i += 0.1;


        context.clear_color(0.5 + 0.5 * f32::sin(0.2 * i), 0.0, 0.0, 1.0);
        context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        context.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, (vertices.len() / 2) as i32);

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

