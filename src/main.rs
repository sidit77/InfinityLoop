#![windows_subsystem = "windows"]

mod opengl;
mod types;
mod app;
mod camera;
mod world;
mod util;

use std::ops::Sub;
use std::time::Duration;
use glam::{Mat4, Quat, Vec2, Vec3};
use crate::app::{Event, EventHandler};
use crate::camera::Camera;
use crate::opengl::{Texture, Buffer, BufferTarget, Context, DataType, PrimitiveType, SetUniform, Shader, ShaderProgram, ShaderType, VertexArray, VertexArrayAttribute, BlendFactor, BlendState, BlendEquation, Framebuffer, TextureType, InternalFormat, MipmapLevels, FramebufferAttachment};
use crate::types::{Color, HexPos};
use crate::world::{World};

struct Game {
    _vertex_buffer: Buffer,
    _index_buffer: Buffer,
    framebuffer: Framebuffer,
    framebuffer_dst: Texture,
    vertex_array: VertexArray,
    textures: Vec<Texture>,
    program: ShaderProgram,
    pp_program: ShaderProgram,
    camera: Camera,
    world: World
}

impl Game {

    fn new(ctx: &Context) -> Self {



        let vertex_array = VertexArray::new(ctx).unwrap();
        ctx.use_vertex_array(&vertex_array);

        let vertex_buffer = Buffer::new(ctx, BufferTarget::Array).unwrap();
        vertex_buffer.set_data::<f32>(&[
            -1., -1., 0., 0.,
             1., -1., 1., 0.,
             1.,  1., 1., 1.,
            -1.,  1., 0., 1.,
        ]);

        let index_buffer = Buffer::new(ctx, BufferTarget::ElementArray).unwrap();
        index_buffer.set_data::<u16>(&[
            0, 1, 2,
            0, 2, 3
        ]);

        vertex_array.set_bindings(&[
            VertexArrayAttribute::Float(DataType::F32, 2, false),
            VertexArrayAttribute::Float(DataType::F32, 2, false)
        ]);

        let program = ShaderProgram::new(ctx, &[
            &Shader::new(ctx, ShaderType::Vertex, include_str!("shader/vertex.glsl")).unwrap(),
            &Shader::new(ctx, ShaderType::Fragment, include_str!("shader/fragment.glsl")).unwrap(),
        ]).unwrap();

        let pp_program = ShaderProgram::new(ctx, &[
            &Shader::new(ctx, ShaderType::Vertex, include_str!("shader/vertex.glsl")).unwrap(),
            &Shader::new(ctx, ShaderType::Fragment, include_str!("shader/pp_fragment.glsl")).unwrap(),
        ]).unwrap();

        let framebuffer_dst = Texture::new(ctx, TextureType::Texture2d(1280, 720), InternalFormat::R8, MipmapLevels::None).unwrap();
        let framebuffer = Framebuffer::new(ctx, &[
            (FramebufferAttachment::Color(0), &framebuffer_dst)
        ]).unwrap();

        let textures = vec![
            Texture::load_png::<&[u8]>(ctx, include_bytes!(concat!(env!("OUT_DIR"), "/hex.png"))).unwrap(),
            Texture::load_png::<&[u8]>(ctx, include_bytes!(concat!(env!("OUT_DIR"), "/tile0.png"))).unwrap(),
            Texture::load_png::<&[u8]>(ctx, include_bytes!(concat!(env!("OUT_DIR"), "/tile01.png"))).unwrap(),
            Texture::load_png::<&[u8]>(ctx, include_bytes!(concat!(env!("OUT_DIR"), "/tile02.png"))).unwrap(),
            Texture::load_png::<&[u8]>(ctx, include_bytes!(concat!(env!("OUT_DIR"), "/tile03.png"))).unwrap(),
            Texture::load_png::<&[u8]>(ctx, include_bytes!(concat!(env!("OUT_DIR"), "/tile012.png"))).unwrap(),
            Texture::load_png::<&[u8]>(ctx, include_bytes!(concat!(env!("OUT_DIR"), "/tile024.png"))).unwrap(),
            Texture::load_png::<&[u8]>(ctx, include_bytes!(concat!(env!("OUT_DIR"), "/tile0134.png"))).unwrap()
        ];

        let camera = Camera {
            scale: 6.0,
            ..Default::default()
        };

        let mut world = World::new(1337);
        world.scramble();

        Self {
            vertex_array,
            _vertex_buffer: vertex_buffer,
            _index_buffer: index_buffer,
            program,
            pp_program,
            camera,
            world,
            textures,
            framebuffer_dst,
            framebuffer
        }
    }
}

impl EventHandler for Game {
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
        for (i, tex) in self.textures.iter().enumerate() {
            ctx.bind_texture(i as u32, tex);
        }
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

        for (hex, conf) in self.world.iter() {
            if !conf.is_empty() {
                self.program.set_uniform_by_name("tex", conf.model() as i32); //
                self.program.set_uniform_by_name("model", Mat4::from_scale_rotation_translation(
                    Vec3::ONE * 1.16,
                    Quat::from_rotation_z(conf.angle().to_radians()),
                    Vec2::from(hex).extend(0.0)));
                ctx.draw_elements_range(PrimitiveType::Triangles, DataType::U16, 0..6);
            }
        }

        ctx.use_framebuffer(None);
        ctx.set_blend_state(None);
        ctx.use_program(&self.pp_program);
        ctx.bind_texture(0, &self.framebuffer_dst);
        self.pp_program.set_uniform_by_name("tex", 0);
        self.pp_program.set_uniform_by_name("camera", Mat4::IDENTITY);
        self.pp_program.set_uniform_by_name("model", Mat4::IDENTITY);
        ctx.draw_elements_range(PrimitiveType::Triangles, DataType::U16, 0..6);
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
                    self.world = World::new(self.world.seed() + 1);
                    self.world.scramble();
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


fn main() {
    app::run(Game::new)
}
