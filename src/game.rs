use crate::camera::Camera;
use web_sys::{WebGl2RenderingContext, WebGlUniformLocation};
use crate::shader::compile_program;
use glam::{Quat, Mat4, Vec2, Vec3Swizzles, Vec3};
use crate::meshes;
use std::ops::Range;

const SIN_FRAC_PI_6: f32 = 0.5;
const COS_FRAC_PI_6: f32 = 0.86602540378;

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

#[derive(Debug, Default, Copy, Clone)]
struct WorldElement {
    rotation: u8
}

#[derive(Debug)]
struct World {
    rows: u32,
    width: u32,
    elements: Box<[WorldElement]>
}

impl World {

    fn new(rows: u32, width: u32) -> Self {
        Self {
            rows,
            width,
            elements: vec![Default::default(); (rows * width + (rows / 2)) as usize].into_boxed_slice()
        }
    }
    fn indices(&self) -> Range<usize> {
        0..self.elements.len()
    }
    fn get_xy(&self, index: usize) -> (u32, u32) {
        let div = index as u32 / (2 * self.width + 1);
        let rem = index as u32 % (2 * self.width + 1);

        if rem < self.width {
            (rem, 2 * div)
        } else {
            (rem - self.width, 2 * div + 1)
        }
    }
    fn get_position(&self, index: usize) -> Vec2{
        let (x, y) = self.get_xy(index);
        let offset = -((y % 2) as f32) * COS_FRAC_PI_6;
        Vec2::new(
            (-0.5 * self.width as f32 - 1.0) + (2.0 * COS_FRAC_PI_6) * x as f32 + offset,
            (-0.5 * self.rows  as f32 - 1.0) + (1.0 + SIN_FRAC_PI_6) * y as f32
        )
    }
    fn get_size(&self) -> (f32, f32) {
        ((2.0 * COS_FRAC_PI_6) * self.width as f32, (1.0 + SIN_FRAC_PI_6) * self.rows as f32)
    }
    fn get_element(&mut self, index: usize) -> &mut WorldElement {
        &mut self.elements[index]
    }
}

pub struct Game {
    gl: WebGl2RenderingContext,
    camera: Camera,
    mvp_location: WebGlUniformLocation,
    color_location: WebGlUniformLocation,
    world: World
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

        let world = World::new(9,5);
        let camera = Camera{
            position: Vec2::new(0.0, 1.0),
            ..Camera::default()
        };

        let mvp_location = gl.get_uniform_location(&program, "mvp").unwrap();
        let color_location = gl.get_uniform_location(&program, "color").unwrap();

        Ok(Self {
            gl,
            camera,
            mvp_location,
            color_location,
            world
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.gl.viewport(0, 0, width as i32, height as i32);

        self.camera.calc_aspect(width, height);
        self.camera.scale = {
            let (w, h) = self.world.get_size();
            f32::max((w / self.camera.aspect) * 0.7, h * 0.6)
        };
        //self.gl.uniform_matrix4fv_with_f32_array(Some(&self.mvp_location), false, &self.camera.to_matrix().to_cols_array());
    }

    pub fn mouse_down(&mut self, x: f32, y: f32) {
        let point = Vec3::new(2.0 * x - 1.0, 2.0 * (1.0 - y) - 1.0, 0.0);
        let point = self.camera.to_matrix().inverse().transform_point3(point);

        for i in self.world.indices() {
            let hex = Hexagon{
                position: self.world.get_position(i),
                rotation: 0.0,
                radius: 1.0
            };
            if hex.contains(point.xy()) {
                self.world.get_element(i).rotation += 1;
            }
        }
    }

    pub fn render(&mut self, _time: f64) {
        self.gl.clear_color(0.2, 0.2, 0.2, 1.0);
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);


        let rng = fastrand::Rng::with_seed(1337);

        self.gl.uniform4f(Some(&self.color_location), 0.9, 0.9, 0.9, 1.0);
        for i in self.world.indices() {
            let obj_mat = self.camera.to_matrix() * Mat4::from_rotation_translation(
                Quat::from_rotation_z(-std::f32::consts::FRAC_PI_3 * self.world.get_element(i).rotation as f32),
                self.world.get_position(i).extend(0.0)
            );
            self.gl.uniform_matrix4fv_with_f32_array(Some(&self.mvp_location), false, &obj_mat.to_cols_array());
            self.gl.uniform4f(Some(&self.color_location), rng.f32(), rng.f32(), rng.f32(), 1.0);
            self.gl.draw_array_range(WebGl2RenderingContext::TRIANGLES, meshes::HEXAGON);
            self.gl.uniform4f(Some(&self.color_location), 0.0, 0.0, 0.0, 1.0);
            self.gl.draw_array_range(WebGl2RenderingContext::TRIANGLES, match rng.u8(0..7) {
                0 => meshes::MODEL1,
                1 => meshes::MODEL2,
                2 => meshes::MODEL3,
                3 => meshes::MODEL4,
                4 => meshes::MODEL5,
                5 => meshes::MODEL6,
                6 => meshes::MODEL7,
                _ => meshes::HEXAGON
            });
        }
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