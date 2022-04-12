use crate::opengl::*;
use anyhow::Result;
use glam::Vec2;

pub struct TextRenderer{
    vertex_array: VertexArray,
    vertex_buffer: Buffer,
    shader: ShaderProgram
}

impl TextRenderer {

    pub fn new(ctx: &Context) -> Result<Self> {
        let vertex_array = VertexArray::new(ctx)?;
        ctx.use_vertex_array(&vertex_array);

        let vertex_buffer = Buffer::new(ctx, BufferTarget::Array)?;
        vertex_array.set_bindings(&vertex_buffer, VertexStepMode::Vertex, &[
            VertexArrayAttribute::Float(0, DataType::F32, 2, false),
            VertexArrayAttribute::Float(1, DataType::F32, 2, false),
        ]);
        vertex_buffer.set_data(&[
            Vec2::new(-0.5, -0.5), Vec2::new(0.0, 0.0),
            Vec2::new( 0.5, -0.5), Vec2::new(1.0, 0.0),
            Vec2::new( 0.5,  0.5), Vec2::new(1.0, 1.0),
        ], BufferUsage::StaticDraw);

        let shader = ShaderProgram::new(ctx, &[
            &Shader::new(ctx, ShaderType::Vertex, include_str!("../shader/text.vert"))?,
            &Shader::new(ctx, ShaderType::Fragment, include_str!("../shader/text.frag"))?
        ])?;

        Ok(Self {
            vertex_array,
            vertex_buffer,
            shader
        })
    }

    pub fn render(&self, ctx: &Context) {
        ctx.use_vertex_array(&self.vertex_array);
        ctx.use_program(&self.shader);
        ctx.draw_arrays(PrimitiveType::Triangles, 0, 3);
    }

}