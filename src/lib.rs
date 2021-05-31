mod shader;
mod camera;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use std::cell::RefCell;
use std::rc::Rc;
use web_sys::{WebGl2RenderingContext, console};
use crate::shader::compile_program;
use crate::camera::Camera;

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    web_sys::window().unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

#[allow(unused_macros)]
macro_rules! console_log {
    ($($t:tt)*) => (web_sys::console::log_1(&format_args!($($t)*).to_string().into()))
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue>{

    let window = web_sys::window().unwrap();
    let canvas = window.document().unwrap().get_element_by_id("canvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>()?;
    let gl = canvas.get_context("webgl2").unwrap().unwrap().dyn_into::<WebGl2RenderingContext>()?;

    let program = compile_program(&gl, &[
        (WebGl2RenderingContext::VERTEX_SHADER, include_str!("shader/vertex.glsl")),
        (WebGl2RenderingContext::FRAGMENT_SHADER, include_str!("shader/fragment.glsl"))
    ])?;
    gl.use_program(Some(&program));

    let vertices: [f32; 6] = [-0.7, -0.7, 0.7, -0.7, 0.0, 0.7];

    let buffer = gl.create_buffer().ok_or("failed to create buffer")?;
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

    unsafe {
        let vert_array = js_sys::Float32Array::view(&vertices);

        gl.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &vert_array,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    gl.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(0);


    let mut camera = Camera {
        scale: 2.0,
        ..Camera::default()
    };

    let mvp_location = gl.get_uniform_location(&program, "mvp").unwrap();

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut i = 0.0;
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        {
            let width = canvas.client_width() as u32;
            let height = canvas.client_height() as u32;

            if width != canvas.width() || height != canvas.height(){
                canvas.set_width(width);
                canvas.set_height(height);
                gl.viewport(0, 0, width as i32, height as i32);

                camera.calc_aspect(width, height);
                gl.uniform_matrix4fv_with_f32_array(Some(&mvp_location), false, &camera.to_matrix().to_cols_array());
            }
        }


        i += 0.1;


        gl.clear_color(0.5 + 0.5 * f32::sin(0.2 * i), 0.0, 0.0, 1.0);
        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        camera.position.x = f32::sin(0.04 * i);
        gl.uniform_matrix4fv_with_f32_array(Some(&mvp_location), false, &camera.to_matrix().to_cols_array());

        gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, (vertices.len() / 2) as i32);

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

