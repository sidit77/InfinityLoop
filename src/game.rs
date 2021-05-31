use crate::camera::Camera;
use web_sys::{WebGl2RenderingContext, WebGlUniformLocation};
use crate::shader::compile_program;
use glam::{Vec3, Quat, Mat4};

const VERTICES: [f32; 6] = [-0.7, -0.7, 0.7, -0.7, 0.0, 0.7];

pub struct Game {
    gl: WebGl2RenderingContext,
    camera: Camera,
    mvp_location: WebGlUniformLocation,
    obj_location: WebGlUniformLocation,
    i: f32,
    point: Vec3
}

impl Game {

    pub fn new(gl: WebGl2RenderingContext) -> Result<Self, String> {
        let program = compile_program(&gl, &[
            (WebGl2RenderingContext::VERTEX_SHADER, include_str!("shader/vertex.glsl")),
            (WebGl2RenderingContext::FRAGMENT_SHADER, include_str!("shader/fragment.glsl"))
        ])?;
        gl.use_program(Some(&program));

        let buffer = gl.create_buffer().ok_or("failed to create buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        unsafe {
            let vert_array = js_sys::Float32Array::view(&VERTICES);

            gl.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        gl.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);


        let camera = Camera {
            scale: 2.0,
            ..Camera::default()
        };

        let mvp_location = gl.get_uniform_location(&program, "cam").unwrap();
        let obj_location = gl.get_uniform_location(&program, "obj").unwrap();

        Ok(Self {
            gl,
            camera,
            mvp_location,
            obj_location,
            i: 0.0,
            point: Vec3::new(0.0,0.0, 0.0)
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.gl.viewport(0, 0, width as i32, height as i32);

        self.camera.calc_aspect(width, height);
        self.gl.uniform_matrix4fv_with_f32_array(Some(&self.mvp_location), false, &self.camera.to_matrix().to_cols_array());
    }

    pub fn mouse_down(&mut self, x: f32, y: f32) {
        let point = Vec3::new(2.0 * x - 1.0, 2.0 * (1.0 - y) - 1.0, 0.0);
        let point = self.camera.to_matrix().inverse().transform_point3(point);
        self.point = point;
    }

    pub fn render(&mut self) {
        self.i += 0.1;


        self.gl.clear_color(0.5 + 0.5 * f32::sin(0.2 * self.i), 0.0, 0.0, 1.0);
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        let obj_mat = Mat4::from_rotation_translation(
            Quat::from_rotation_z(0.04 * self.i),
            self.point);
        self.gl.uniform_matrix4fv_with_f32_array(Some(&self.obj_location), false, &obj_mat.to_cols_array());

        self.gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, (VERTICES.len() / 2) as i32);
    }

}