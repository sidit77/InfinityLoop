use crate::camera::Camera;
use web_sys::{WebGl2RenderingContext, WebGlUniformLocation};
use crate::shader::compile_program;
use glam::{Quat, Mat4, Vec2, Vec3Swizzles, Vec3};
use crate::meshes;
use crate::intersection::Hexagon;
use std::ops::Range;
use crate::world::World;
use css_color_parser::Color;

pub struct GameStyle {
    pub foreground: Color,
    pub background: Color
}

pub struct Game {
    gl: WebGl2RenderingContext,
    style: GameStyle,
    camera: Camera,
    mvp_location: WebGlUniformLocation,
    color_location: WebGlUniformLocation,
    world: World,
    rng: fastrand::Rng,
}

impl Game {

    pub fn new(gl: WebGl2RenderingContext, style: GameStyle) -> Result<Self, String> {
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

        let world = World::from_seed(1337);
        let camera = Camera{
            position: Vec2::new(0.0, 1.0),
            ..Camera::default()
        };

        let mvp_location = gl.get_uniform_location(&program, "mvp").unwrap();
        let color_location = gl.get_uniform_location(&program, "color").unwrap();

        Ok(Self {
            gl,
            style,
            camera,
            mvp_location,
            color_location,
            world,
            rng: fastrand::Rng::with_seed(1337)
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

    pub fn new_level(&mut self){
        self.world = World::from_seed(self.rng.u64(0..9999));
    }

    pub fn scramble_level(&mut self) {
        self.world.scramble(&self.rng);
    }

    pub fn mouse_down(&mut self, x: f32, y: f32) {
        let point = Vec3::new(2.0 * x - 1.0, 2.0 * (1.0 - y) - 1.0, 0.0);
        let point = self.camera.to_matrix().inverse().transform_point3(point);

        for i in self.world.indices() {
            let position = self.world.get_position(i);
            if let Some(elem) = self.world.get_element(i) {
                let hex = Hexagon{
                    position,
                    rotation: 0.0,
                    radius: 1.0
                };
                if hex.contains(point.xy()) {
                    elem.rotation += 1;
                }
            }

        }
    }

    pub fn render(&mut self, _time: f64) {
        {
            let c = self.style.background.as_f32();
            self.gl.clear_color(c[0], c[1], c[2], c[3]);
        }
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        //let rng = fastrand::Rng::with_seed(1337);

        self.gl.uniform4fv_with_f32_array(Some(&self.color_location), &self.style.foreground.as_f32());
        for i in self.world.indices() {
            let position = self.world.get_position(i);
            if let Some(elem) = self.world.get_element(i).as_ref() {
                let obj_mat = self.camera.to_matrix() * Mat4::from_rotation_translation(
                    Quat::from_rotation_z(-std::f32::consts::FRAC_PI_3 * elem.rotation as f32),
                    position.extend(0.0)
                );
                self.gl.uniform_matrix4fv_with_f32_array(Some(&self.mvp_location), false, &obj_mat.to_cols_array());
                //self.gl.uniform4f(Some(&self.color_location), rng.f32(), rng.f32(), rng.f32(), 1.0);
                //self.gl.draw_array_range(WebGl2RenderingContext::TRIANGLES, meshes::HEXAGON);
                //self.gl.uniform4f(Some(&self.color_location), 0.0, 0.0, 0.0, 1.0);
                self.gl.draw_array_range(WebGl2RenderingContext::TRIANGLES, elem.tile_type.model());
            }
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

trait AsF32 {
    fn as_f32(&self) -> [f32; 4];
}

impl AsF32 for Color {
    fn as_f32(&self) -> [f32; 4] {
        [
            self.r as f32 / u8::MAX as f32,
            self.g as f32 / u8::MAX as f32,
            self.b as f32 / u8::MAX as f32,
            self.a
        ]
    }
}