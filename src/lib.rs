mod opengl;
mod types;
mod app;
mod camera;
mod world;
mod util;
mod renderer;

use std::ops::{Add, Rem, Sub};
use std::rc::Rc;
use glam::Vec2;
use crate::app::{AppContext, Bundle, Event, Game};
use crate::camera::Camera;
use crate::opengl::{Texture, Buffer, BufferTarget, Context, DataType, PrimitiveType, SetUniform, Shader, ShaderProgram, ShaderType, Framebuffer, TextureType, InternalFormat, MipmapLevels, FramebufferAttachment};
use crate::types::{Color, HexPos, Rgba};
use crate::world::{World};
use crate::renderer::{RenderableWorld, TileRenderResources};

pub mod export {
    pub use crate::opengl::Context;
    pub use crate::app::{GlowContext, Application, AppContext, Result};
}

#[derive(Clone)]
pub struct InfinityLoopBundle {
    world: World,
    camera: Camera,
    time: f32
}

impl Bundle for InfinityLoopBundle {
    fn new() -> anyhow::Result<Self> {
        let camera = Camera {
            scale: 6.0,
            ..Default::default()
        };

        let mut world = World::new(1337);
        world.scramble();

        Ok(Self {
            world,
            camera,
            time: 0.0
        })
    }
}

pub struct InfinityLoop {
    framebuffer: Framebuffer,
    framebuffer_dst: Texture,
    pp_program: ShaderProgram,
    camera: Camera,
    world: RenderableWorld,
    time: f32
}

impl Game for InfinityLoop {
    type Bundle = InfinityLoopBundle;

    fn resume<A: AppContext>(ctx: &A, bundle: Self::Bundle) -> anyhow::Result<Self> {
        let pp_program = ShaderProgram::new(&ctx, &[
            &Shader::new(&ctx, ShaderType::Vertex, include_str!("shader/postprocess.vert"))?,
            &Shader::new(&ctx, ShaderType::Fragment, include_str!("shader/postprocess.frag"))?,
        ])?;
        ctx.use_program(&pp_program);
        ctx.set_uniform(&pp_program.get_uniform("tex")?, 0);

        let (width, height) = ctx.screen_size();
        ctx.viewport(0, 0, width as i32, height as i32);
        let framebuffer_dst = Texture::new(&ctx, TextureType::Texture2d(width, height), InternalFormat::R8, MipmapLevels::None)?;
        let framebuffer = Framebuffer::new(&ctx, &[
            (FramebufferAttachment::Color(0), &framebuffer_dst)
        ])?;

        let camera = Camera {
            aspect: width as f32 / height as f32,
            ..bundle.camera
        };

        let resources = Rc::new(TileRenderResources::new(&ctx)?);

        let world = RenderableWorld::new(&ctx, resources, bundle.world)?;

        Ok(Self {
            pp_program,
            camera,
            world,
            framebuffer_dst,
            framebuffer,
            time: bundle.time
        })
    }

    fn suspend<A: AppContext>(self, _ctx: &A) -> Self::Bundle {
        Self::Bundle {
            world: self.world.into(),
            camera: self.camera,
            time: self.time
        }
    }

    fn event<A: AppContext>(&mut self, ctx: &A, event: Event) -> anyhow::Result<bool> {
        let mut camera_update = false;
        match event {
            Event::Draw(delta) => {
                self.time = self.time.add(delta.as_secs_f32() * 0.5).rem(10.0); //6.4;//
                ctx.clear(Rgba::new(23,23,23,255));
                self.world.update(delta);

                ctx.use_framebuffer(&self.framebuffer);
                self.world.render(&ctx, &self.camera);

                ctx.use_framebuffer(None);
                ctx.set_blend_state(None);
                ctx.use_program(&self.pp_program);
                ctx.set_uniform(&self.pp_program.get_uniform("time")?, self.time); //
                ctx.set_uniform(&self.pp_program.get_uniform("inv_camera")?, self.camera.to_matrix().inverse());
                ctx.set_uniform(&self.pp_program.get_uniform("pxRange")?, ctx.screen_height() as f32 / (2.0 * self.camera.scale));
                ctx.bind_texture(0, &self.framebuffer_dst);
                ctx.draw_arrays(PrimitiveType::TriangleStrip, 0, 4);
            },
            Event::Resize(width, height) => {
                assert!(width != 0 && height != 0);
                ctx.viewport(0, 0, width as i32, height as i32);
                self.camera.aspect = width as f32 / height as f32;
                self.framebuffer_dst = Texture::new(&ctx, TextureType::Texture2d(width, height),
                                                    InternalFormat::R8, MipmapLevels::None)?;
                self.framebuffer.update_attachments(&[(FramebufferAttachment::Color(0), &self.framebuffer_dst)])?;
                camera_update = true;
            },
            Event::Click(pos) => {
                let pt = self.camera.to_world_coords(pos).into();
                self.world.try_rotate(pt);
                if self.world.is_completed() {
                    let mut new_world = World::new(self.world.seed() + 1);
                    new_world.scramble();
                    self.world.reinitialize(new_world);
                }
            },
            Event::Zoom(center, amount) => {
                let camera = &mut self.camera;
                let old = camera.to_world_coords(center);
                camera.scale = camera.scale.sub(amount * (camera.scale / 10.0)).max(1.0);
                let new = camera.to_world_coords(center);
                camera.position += old - new;
                camera_update = true;
            }
            Event::Drag(delta) => {
                self.camera.position += self.camera.to_world_coords(-delta) - self.camera.to_world_coords(Vec2::ZERO);
                camera_update = true;
            }
        }
        Ok(camera_update || self.world.update_required())
    }
}

