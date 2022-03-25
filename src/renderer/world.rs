use bytemuck::{Pod, Zeroable};
use glam::{Mat3, Vec2};
use crate::{BlendEquation, BlendFactor, BlendState, Buffer, BufferTarget, Camera, Color, Context, DataType, HexPos, PrimitiveType, SetUniform, Shader, ShaderProgram, ShaderType, Texture, VertexArray, VertexArrayAttribute, VertexStepMode, World};
use crate::world::generate_tile_texture;

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Instance {
    model: Mat3,
    texture: u32
}

pub struct RenderableWorld {
    shader: ShaderProgram,
    textures: Texture,
    vertex_array: VertexArray,
    instance_buffer: Buffer,
    world: World
}

impl RenderableWorld {

    pub fn new(ctx: &Context, world: World) -> Result<Self, String> {
        let vertex_array = VertexArray::new(ctx)?;
        ctx.use_vertex_array(&vertex_array);

        let instance_buffer = Buffer::new(ctx, BufferTarget::Array)?;
        vertex_array.set_bindings(&instance_buffer, VertexStepMode::Instance, &[
            VertexArrayAttribute::Float(0, DataType::F32, 3, false),
            VertexArrayAttribute::Float(1, DataType::F32, 3, false),
            VertexArrayAttribute::Float(2, DataType::F32, 3, false),
            VertexArrayAttribute::Integer(3, DataType::U32, 1)
        ]);

        let shader = ShaderProgram::new(ctx, &[
            &Shader::new(ctx, ShaderType::Vertex, include_str!("../shader/tiles.vert"))?,
            &Shader::new(ctx, ShaderType::Fragment, include_str!("../shader/tiles.frag"))?,
        ])?;

        let textures = generate_tile_texture(ctx)?;

        Ok(Self {
            shader,
            textures,
            vertex_array,
            instance_buffer,
            world
        })
    }

    pub fn render(&self, ctx: &Context, camera: &Camera) {
        ctx.use_vertex_array(&self.vertex_array);

        ctx.clear(Color::new(0, 0, 0, 255));
        ctx.set_blend_state(BlendState {
            src: BlendFactor::One,
            dst: BlendFactor::One,
            equ: BlendEquation::Max
        });
        ctx.use_program(&self.shader);
        ctx.bind_texture(0, &self.textures);
        self.shader.set_uniform_by_name("tex", 0);
        self.shader.set_uniform_by_name("camera", camera.to_matrix());

        let mut instances = Vec::new();
        for (hex, conf) in self.world.iter() {
            if !conf.is_empty() {
                instances.push(Instance {
                    model: Mat3::from_scale_angle_translation(
                        Vec2::ONE * 1.16,
                        conf.angle().to_radians(),
                        hex.into()
                    ),
                    texture: conf.model() as u32
                });
            }
        }

        self.instance_buffer.set_data(instances.as_slice());
        ctx.draw_arrays_instanced(PrimitiveType::TriangleStrip, 0, 4, instances.len() as i32);

    }

    pub fn try_rotate(&mut self, pos: HexPos) -> bool {
        self.world.try_rotate(pos)
    }

    pub fn is_completed(&self) -> bool {
        self.world.is_completed()
    }

}