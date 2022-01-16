mod opengl;
mod types;
mod meshes;
mod app;
mod camera;

use std::time::Duration;
use glam::Mat4;
use glow::HasContext;
use crate::app::{Event, EventHandler};
use crate::camera::Camera;
use crate::opengl::{Buffer, BufferTarget, Context, DataType, PrimitiveType, Shader, ShaderProgram, ShaderType, VertexArray, VertexArrayAttribute};
use crate::types::Color;

struct Game {
    _vertex_buffer: Buffer,
    _index_buffer: Buffer,
    vertex_array: VertexArray,
    program: ShaderProgram,
    camera: Camera
}

impl Game {

    fn new(ctx: &Context) -> Self {
        let vertex_array = VertexArray::new(&ctx).unwrap();
        ctx.use_vertex_array(&vertex_array);

        let vertex_buffer = Buffer::new(&ctx, BufferTarget::Array).unwrap();
        vertex_buffer.set_data(meshes::VERTICES);

        let index_buffer = Buffer::new(&ctx, BufferTarget::ElementArray).unwrap();
        index_buffer.set_data(meshes::INDICES);

        vertex_array.set_bindings(&[VertexArrayAttribute::Float(DataType::F32, 2, false)]);

        let program = ShaderProgram::new(&ctx, &[
            &Shader::new(&ctx, ShaderType::Vertex, include_str!("shader/vertex.glsl")).unwrap(),
            &Shader::new(&ctx, ShaderType::Fragment, include_str!("shader/fragment.glsl")).unwrap(),
        ]).unwrap();

        let camera = Camera::default();

        Self {
            vertex_array,
            _vertex_buffer: vertex_buffer,
            _index_buffer: index_buffer,
            program,
            camera
        }
    }

}

impl EventHandler for Game {
    fn draw(&mut self, ctx: &Context, delta: Duration) {
        ctx.clear(Color::new(46, 52, 64, 255));

        ctx.use_vertex_array(&self.vertex_array);
        ctx.use_program(&self.program);

        unsafe {
            ctx.raw().uniform_matrix_4_f32_slice(self.program.get_uniform_name("camera").as_ref(), false, &self.camera.to_matrix().to_cols_array());
            ctx.raw().uniform_matrix_4_f32_slice(self.program.get_uniform_name("model").as_ref(), false, &Mat4::IDENTITY.to_cols_array());
        }


        ctx.draw_elements_range(PrimitiveType::Triangles, DataType::U16, meshes::MODEL7);
    }

    fn event(&mut self, event: app::Event) {
        match event {
            Event::WindowResize(width, height) => {
                self.camera.aspect = width / height;
            }
            Event::Click(_) => {}
            Event::Drag(_) => {}
            Event::DragEnd(_) => {}
            Event::Touch => {}
            Event::Zoom(_, _) => {}
        }
    }
}


fn main() {
    app::run(|ctx| Game::new(ctx))
}
