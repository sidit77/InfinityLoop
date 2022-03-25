mod opengl;
mod types;
mod app;
mod camera;
mod world;
mod util;

use std::ops::Sub;
use std::time::Duration;
use glam::{Mat3, Vec2};
use bytemuck::{Pod, Zeroable};
use crate::app::Event;
use crate::camera::Camera;
use crate::opengl::{Texture, Buffer, BufferTarget, Context, DataType, PrimitiveType, SetUniform, Shader, ShaderProgram, ShaderType, VertexArray, VertexArrayAttribute, BlendFactor, BlendState, BlendEquation, Framebuffer, TextureType, InternalFormat, MipmapLevels, FramebufferAttachment, VertexStepMode};
use crate::types::{Color, HexPos};
use crate::world::{World};

pub use crate::app::{Game, GlowContext, Platform, PlatformWindow};

#[derive(Debug, Copy, Clone, Pod, Zeroable)]
#[repr(C)]
struct Instance {
    model: Mat3,
    texture: u32
}

pub struct InfinityLoop {
    instance_buffer: Buffer,
    framebuffer: Framebuffer,
    framebuffer_dst: Texture,
    vertex_array: VertexArray,
    textures: Texture,
    program: ShaderProgram,
    pp_program: ShaderProgram,
    camera: Camera,
    world: World
}

impl Game for InfinityLoop {
    fn initialize(ctx: &Context) -> Self {
        let vertex_array = VertexArray::new(ctx).unwrap();
        ctx.use_vertex_array(&vertex_array);

        let instance_buffer = Buffer::new(ctx, BufferTarget::Array).unwrap();
        vertex_array.set_bindings(&instance_buffer, VertexStepMode::Instance, &[
            VertexArrayAttribute::Float(0, DataType::F32, 3, false),
            VertexArrayAttribute::Float(1, DataType::F32, 3, false),
            VertexArrayAttribute::Float(2, DataType::F32, 3, false),
            VertexArrayAttribute::Integer(3, DataType::U32, 1)
        ]);

        let program = ShaderProgram::new(ctx, &[
            &Shader::new(ctx, ShaderType::Vertex, include_str!("shader/tiles.vert")).unwrap(),
            &Shader::new(ctx, ShaderType::Fragment, include_str!("shader/tiles.frag")).unwrap(),
        ]).unwrap();

        let pp_program = ShaderProgram::new(ctx, &[
            &Shader::new(ctx, ShaderType::Vertex, include_str!("shader/postprocess.vert")).unwrap(),
            &Shader::new(ctx, ShaderType::Fragment, include_str!("shader/postprocess.frag")).unwrap(),
        ]).unwrap();

        let framebuffer_dst = Texture::new(ctx, TextureType::Texture2d(1280, 720), InternalFormat::R8, MipmapLevels::None).unwrap();
        let framebuffer = Framebuffer::new(ctx, &[
            (FramebufferAttachment::Color(0), &framebuffer_dst)
        ]).unwrap();

        let textures = world::generate_tile_texture(ctx).unwrap();

        let camera = Camera {
            scale: 6.0,
            ..Default::default()
        };

        let mut world = World::new(1337);
        world.scramble();

        Self {
            vertex_array,
            program,
            pp_program,
            camera,
            world,
            textures,
            framebuffer_dst,
            framebuffer,
            instance_buffer
        }
    }

    fn draw(&mut self, ctx: &Context, _delta: Duration) {

        ctx.use_vertex_array(&self.vertex_array);
        ctx.use_framebuffer(&self.framebuffer);
        ctx.clear(Color::new(0, 0, 0, 255));
        ctx.set_blend_state(BlendState {
            src: BlendFactor::One,
            dst: BlendFactor::One,
            equ: BlendEquation::Max
        });
        ctx.use_program(&self.program);
        ctx.bind_texture(0, &self.textures);
        self.program.set_uniform_by_name("tex", 0);
        //ctx.bind_texture(0, &self.texture);
        self.program.set_uniform_by_name("camera", self.camera.to_matrix());


        //let tile = TileType::Tile0134;
        //let pos = HexPos::CENTER;
        //self.program.set_uniform_by_name("model", Mat4::from_translation(Vec2::from(pos).extend(0.0)));
        //ctx.draw_elements_range(PrimitiveType::Triangles, DataType::U16, 0..6);
//
        //for (i, n) in pos.neighbors().enumerate(){
        //    if tile.endings()[i] {
        //        self.program.set_uniform_by_name("model", Mat4::from_translation(Vec2::from(n).extend(0.0)));
        //        ctx.draw_elements_range(PrimitiveType::Triangles, DataType::U16, 0..6);
        //    }
        //}

        //self.program.set_uniform_by_name("tex_id", 6i32); //

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


        ctx.use_framebuffer(None);
        ctx.set_blend_state(None);
        ctx.use_program(&self.pp_program);
        ctx.bind_texture(0, &self.framebuffer_dst);
        self.pp_program.set_uniform_by_name("tex", 0);
        ctx.draw_arrays(PrimitiveType::TriangleStrip, 0, 4);
    }

    fn event(&mut self, ctx: &Context, event: app::Event) {
        match event {
            Event::WindowResize(width, height) => {
                self.camera.aspect = width / height;
                self.framebuffer_dst = Texture::new(ctx, TextureType::Texture2d(width as u32, height as u32),
                                                    InternalFormat::R8, MipmapLevels::None).unwrap();
                self.framebuffer.update_attachments(&[(FramebufferAttachment::Color(0), &self.framebuffer_dst)]).unwrap();
            }
            Event::Click(pos) => {
                let pt = self.camera.to_world_coords(pos).into();
                self.world.try_rotate(pt);
                if self.world.is_completed() {
                    //self.world = World::new(self.world.seed() + 1);
                    //self.world.scramble();
                    println!("Well done!");
                }

            },
            Event::Zoom(center, amount) => {
                let camera = &mut self.camera;
                let old = camera.to_world_coords(center);
                camera.scale = camera.scale.sub(amount * (camera.scale / 10.0)).max(1.0);
                let new = camera.to_world_coords(center);
                camera.position += old - new;
            }
            Event::Drag(delta) => {
                self.camera.position += self.camera.to_world_coords(-delta.absolute()) - self.camera.to_world_coords(Vec2::ZERO);
            },
            Event::DragEnd(_) => {},
            Event::DragStart(_) => {}
        }
    }
}

