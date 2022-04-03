mod opengl;
mod types;
mod app;
mod camera;
mod world;
mod util;
mod renderer;

use std::ops::{Sub};
use std::rc::Rc;
use glam::Vec2;
use crate::app::{AppContext, Bundle, Event, Game};
use crate::camera::Camera;
use crate::opengl::{Texture, Buffer, BufferTarget, Context, DataType, Framebuffer, TextureType, InternalFormat, MipmapLevels, FramebufferAttachment};
use crate::types::{Color, HexPos, Rgba};
use crate::world::{World};
use crate::renderer::{GameRenderer, GameState, RenderableWorld, TileRenderResources};

pub mod export {
    pub use crate::opengl::Context;
    pub use crate::app::{GlowContext, Application, AppContext, Result};
}

#[derive(Clone)]
pub struct InfinityLoopBundle {
    world: World,
    camera: Camera,
    state: GameState
}

impl Bundle for InfinityLoopBundle {
    fn new() -> anyhow::Result<Self> {
        let camera = Camera {
            scale: 6.0,
            ..Default::default()
        };

        let mut world = World::new(1337);
        world.scramble();

        let state = match world.is_completed() {
            true => GameState::Ending(Vec2::ZERO, f32::INFINITY),
            false => GameState::InProgress
        };
        
        Ok(Self {
            world,
            camera,
            state
        })
    }
}

pub struct InfinityLoop {
    framebuffer: Framebuffer,
    framebuffer_dst: Texture,
    renderer: GameRenderer,
    camera: Camera,
    world: RenderableWorld,
    state: GameState
}

impl Game for InfinityLoop {
    type Bundle = InfinityLoopBundle;

    fn resume<A: AppContext>(ctx: &A, bundle: Self::Bundle) -> anyhow::Result<Self> {
        let renderer = GameRenderer::new(&ctx)?;

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
            renderer,
            camera,
            world,
            framebuffer_dst,
            framebuffer,
            state: bundle.state
        })
    }

    fn suspend<A: AppContext>(self, _ctx: &A) -> Self::Bundle {
        Self::Bundle {
            world: self.world.into(),
            camera: self.camera,
            state: self.state
        }
    }

    fn event<A: AppContext>(&mut self, ctx: &A, event: Event) -> anyhow::Result<bool> {
        let mut camera_update = false;
        match event {
            Event::Draw(delta) => {
                self.state.update(delta, self.world.update_required());
                self.world.update(delta);

                ctx.clear(Rgba::new(23,23,23,255));

                ctx.use_framebuffer(&self.framebuffer);
                self.world.render(&ctx, &self.camera);

                ctx.use_framebuffer(None);
                self.renderer.render(ctx, self.state, &self.camera, &self.framebuffer_dst)?;
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
            Event::Click(pos) => match self.state{
                GameState::InProgress => {
                    let pt = self.camera.to_world_coords(pos).into();
                    self.world.try_rotate(pt);
                    if self.world.is_completed() {
                        self.state.set(GameState::WaitingForEnd(pt.into()));
                    }
                }
                GameState::Ending(_, _) => {
                    self.state.set(GameState::Ended);
                    camera_update = true;
                },
                GameState::Ended => {
                    let mut new_world = World::new(self.world.seed() + 1);
                    new_world.scramble();
                    self.world.reinitialize(new_world);
                    self.state.set(GameState::InProgress);
                    camera_update = true;
                }
                _ => {}
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
        Ok(camera_update || self.world.update_required() || self.state.is_animated())
    }
}

