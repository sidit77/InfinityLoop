use crate::camera::Camera;
use web_sys::{WebGl2RenderingContext, WebGlUniformLocation};
use crate::shader::compile_program;
use glam::{Quat, Mat4, Vec2, Vec3Swizzles, Vec3};
use bytemuck::{Pod, Zeroable};
use crate::meshes;
use std::ops::Range;

struct Triangle(Vec2, Vec2, Vec2);

impl Triangle {
    fn contains(&self, point: Vec2) -> bool{
        let d_x = point.x - self.2.x;
        let d_y = point.y - self.2.y;
        let d_x21 = self.2.x - self.1.x;
        let d_y12 = self.1.y - self.2.y;
        let d = d_y12 * (self.0.x - self.2.x) + d_x21 * (self.0.y - self.2.y);
        let s = d_y12 * d_x + d_x21 * d_y;
        let t = (self.2.y - self.0.y) * d_x + (self.0.x - self.2.x) * d_y;
        if d < 0.0 {
            return s <= 0.0 && t <= 0.0 && s + t >= d;
        }
        return s >= 0.0 && t >= 0.0 && s+t <= d;
    }
}

#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Pod, Zeroable)]
struct Vertex {
    x: f32,
    y: f32
}

struct Hexagon {
    position: Vec2,
    rotation: f32,
    radius: f32
}

impl Default for Hexagon {
    fn default() -> Self {
        Self {
            position: Vec2::new(0.0,0.0),
            rotation: 0.0,
            radius: 1.0
        }
    }
}

impl Hexagon {

    fn contains(&self, point: Vec2) -> bool {
        if (point - self.position).length_squared() > self.radius {
            return false;
        }

        let from_id = |i: u32|{
            let (sin, cos) = f32::sin_cos(-self.rotation + std::f32::consts::FRAC_PI_3 * i as f32);
            Vec2::new(self.position.x + self.radius * sin, self.position.y + self.radius * cos)
        };

        for i in 0u32..4 {
            if Triangle(from_id(0), from_id(i + 1), from_id(i + 2)).contains(point){
                return true
            }
        }
        false
    }

}

pub struct Game {
    gl: WebGl2RenderingContext,
    camera: Camera,
    mvp_location: WebGlUniformLocation,
    obj_location: WebGlUniformLocation,
    color_location: WebGlUniformLocation,
    hexagon: Hexagon
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

        gl.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            meshes::VERTEX_DATA,
            WebGl2RenderingContext::STATIC_DRAW,
        );
        gl.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);


        //console_log!("{:?}", crate::renderer::meshes::MODEL1);

        let camera = Camera {
            scale: 2.0,
            ..Camera::default()
        };

        let mvp_location = gl.get_uniform_location(&program, "cam").unwrap();
        let obj_location = gl.get_uniform_location(&program, "obj").unwrap();
        let color_location = gl.get_uniform_location(&program, "color").unwrap();

        Ok(Self {
            gl,
            camera,
            mvp_location,
            obj_location,
            color_location,
            hexagon: Default::default()
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
        if self.hexagon.contains(point.xy()){
            self.hexagon.rotation -= 0.1;
        }
    }

    pub fn render(&mut self, _time: f64) {
        self.gl.clear_color(0.2, 0.2, 0.2, 1.0);
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        //let obj_mat = Mat4::from_scale_rotation_translation(
        //    Vec3::new(self.hexagon.radius, self.hexagon.radius, self.hexagon.radius),
        //    Quat::from_rotation_z(self.hexagon.rotation),
        //    self.hexagon.position.extend(0.0));
        let obj_mat = Mat4::IDENTITY;
        self.gl.uniform_matrix4fv_with_f32_array(Some(&self.obj_location), false, &obj_mat.to_cols_array());
        self.gl.uniform4f(Some(&self.color_location), 1.0, 0.8, 0.8, 1.0);

        self.gl.draw_array_range(WebGl2RenderingContext::TRIANGLES, meshes::HEXAGON);

        self.gl.uniform4f(Some(&self.color_location), 0.8, 1.0, 0.8, 1.0);
        self.gl.draw_array_range(WebGl2RenderingContext::TRIANGLES, meshes::MODEL1);
    }

}

trait DrawRange {
    fn draw_array_range(&self, mode: u32, range: std::ops::Range<i32>);
}

impl DrawRange for WebGl2RenderingContext {
    fn draw_array_range(&self, mode: u32, range: Range<i32>) {
        self.draw_arrays(mode, range.start, range.len() as i32);
    }
}