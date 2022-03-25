mod opengl;
mod types;
mod app;
mod camera;
mod world;
mod util;
mod renderer;

use std::ops::Sub;
use std::time::Duration;
use glam::Vec2;
use crate::app::Event;
use crate::camera::Camera;
use crate::opengl::{Texture, Buffer, BufferTarget, Context, DataType, PrimitiveType, SetUniform, Shader, ShaderProgram, ShaderType, Framebuffer, TextureType, InternalFormat, MipmapLevels, FramebufferAttachment};
use crate::types::{Color, HexPos};
use crate::world::{World};

pub use crate::app::{Game, GlowContext, Platform, PlatformWindow};
use crate::renderer::RenderableWorld;

pub struct InfinityLoop {
    framebuffer: Framebuffer,
    framebuffer_dst: Texture,
    pp_program: ShaderProgram,
    camera: Camera,
    world: RenderableWorld
}

impl Game for InfinityLoop {
    fn initialize(ctx: &Context) -> Self {

        let pp_program = ShaderProgram::new(ctx, &[
            &Shader::new(ctx, ShaderType::Vertex, include_str!("shader/postprocess.vert")).unwrap(),
            &Shader::new(ctx, ShaderType::Fragment, include_str!("shader/postprocess.frag")).unwrap(),
        ]).unwrap();

        let framebuffer_dst = Texture::new(ctx, TextureType::Texture2d(1280, 720), InternalFormat::R8, MipmapLevels::None).unwrap();
        let framebuffer = Framebuffer::new(ctx, &[
            (FramebufferAttachment::Color(0), &framebuffer_dst)
        ]).unwrap();

        let camera = Camera {
            scale: 6.0,
            ..Default::default()
        };

        let mut world = World::new(1337);
        world.scramble();
        let world = RenderableWorld::new(ctx, world).unwrap();

        Self {
            pp_program,
            camera,
            world,
            framebuffer_dst,
            framebuffer
        }
    }

    fn draw(&mut self, ctx: &Context, _delta: Duration) {
        ctx.use_framebuffer(&self.framebuffer);
        self.world.render(ctx, &self.camera);

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

