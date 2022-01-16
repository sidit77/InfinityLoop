mod opengl;
mod types;
mod meshes;
mod app;

use std::time::Duration;
use crate::app::{Event, EventHandler};
use crate::opengl::{Buffer, BufferTarget, Context, DataType, PrimitiveType, Shader, ShaderProgram, ShaderType, VertexArray, VertexArrayAttribute};
use crate::types::Color;

struct Game {
    _vertex_buffer: Buffer,
    _index_buffer: Buffer,
    vertex_array: VertexArray,
    program: ShaderProgram
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

        let (vertex_shader_source, fragment_shader_source) = (
            r#"#version 300 es
        layout(location = 0) in vec2 pos;
        out vec2 vert;
        void main() {
            vert = pos * 0.3;
            gl_Position = vec4(vert - 0.5, 0.0, 1.0);
        }"#,
            r#"#version 300 es
        precision mediump float;
        in vec2 vert;
        out vec4 color;
        void main() {
            color = vec4(vert, 0.5, 1.0);
        }"#,
        );

        let program = ShaderProgram::new(&ctx, &[
            &Shader::new(&ctx, ShaderType::Vertex, vertex_shader_source).unwrap(),
            &Shader::new(&ctx, ShaderType::Fragment, fragment_shader_source).unwrap(),
        ]).unwrap();

        Self {
            vertex_array,
            _vertex_buffer: vertex_buffer,
            _index_buffer: index_buffer,
            program
        }
    }

}

impl EventHandler for Game {
    fn draw(&mut self, ctx: &Context, delta: Duration) {
        ctx.use_program(&self.program);
        ctx.use_vertex_array(&self.vertex_array);

        ctx.clear(Color::new(46, 52, 64, 255));
        ctx.draw_elements_range(PrimitiveType::Triangles, DataType::U16, meshes::MODEL7);
    }

    fn event(&mut self, event: app::Event) {
        match event {
            Event::WindowResize(_, _) => {}
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
