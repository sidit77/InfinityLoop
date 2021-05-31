use std::cell::{RefCell, Cell};
use std::rc::Rc;

use glam::{Mat4, Vec3, Vec2, Quat};
use wasm_bindgen::JsCast;
use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext;

use crate::camera::Camera;
use crate::shader::compile_program;

mod shader;
mod camera;

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

    let last_click = Rc::new(Cell::new(Vec2::new(0.0,0.0)));
    {
        let last_click = last_click.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            last_click.set(Vec2::new(event.client_x() as f32, event.client_y() as f32));
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

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

    let mvp_location = gl.get_uniform_location(&program, "cam").unwrap();
    let obj_location = gl.get_uniform_location(&program, "obj").unwrap();

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

        let point = last_click.get();
        let point = Vec2::new(point.x / canvas.width() as f32, point.y / canvas.height() as f32);
        let point = Vec2::new(2.0 * point.x - 1.0, 2.0 * (1.0 - point.y) - 1.0);
        let point = camera.to_matrix().inverse().transform_point3(point.extend(0.0));
        let obj_mat = Mat4::from_rotation_translation(
            Quat::from_rotation_z(0.04 * i),
            point);
        gl.uniform_matrix4fv_with_f32_array(Some(&obj_location), false, &obj_mat.to_cols_array());

        gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, (vertices.len() / 2) as i32);

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}

