mod opengl;
mod types;
mod app;
mod camera;
mod world;
mod util;
mod renderer;

use std::rc::Rc;
use artery_font::ArteryFont;
use glam::Vec2;
use crate::app::{AppContext, Bundle, Event, Game};
use crate::camera::{AnimatedCamera, Camera};
use crate::types::{Color, HexPos, Rgba};
use crate::world::{World};
use crate::renderer::{GameRenderer, GameState, RenderableWorld, TextRenderer, TileRenderResources};

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

        let mut world = World::new(1);
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
    renderer: GameRenderer,
    camera: AnimatedCamera,
    world: RenderableWorld,
    old_world: RenderableWorld,
    text_renderer: TextRenderer,
    state: GameState
}

impl Game for InfinityLoop {
    type Bundle = InfinityLoopBundle;

    fn resume<A: AppContext>(ctx: &A, bundle: Self::Bundle) -> anyhow::Result<Self> {
        let renderer = GameRenderer::new(ctx)?;

        let (width, height) = ctx.screen_size();
        ctx.viewport(0, 0, width as i32, height as i32);

        let camera = Camera {
            aspect: width as f32 / height as f32,
            ..bundle.camera
        }.into();

        let resources = Rc::new(TileRenderResources::new(ctx)?);

        let old_world = RenderableWorld::new(ctx, resources.clone(),
                                             World::new(bundle.world.seed() - 1), (width, height))?;
        let world = RenderableWorld::new(ctx, resources, bundle.world, (width, height))?;

        let mut text_renderer = TextRenderer::new(ctx, &ArteryFont::read(include_bytes!("font/test.arfont").as_slice())?)?;
        text_renderer.set_text(&format!("Level {}", world.seed()));
        
        Ok(Self {
            renderer,
            camera,
            world,
            old_world,
            text_renderer,
            state: bundle.state
        })
    }

    fn suspend<A: AppContext>(self, _ctx: &A) -> Self::Bundle {
        Self::Bundle {
            world: self.world.into(),
            camera: self.camera.into(),
            state: self.state
        }
    }

    fn event<A: AppContext>(&mut self, ctx: &A, event: Event) -> anyhow::Result<bool> {
        let mut camera_update = false;
        match event {
            Event::Draw(delta) => {
                self.camera.update(delta);
                self.state.update(delta, self.world.update_required());
                self.world.update(delta);

                ctx.clear(Rgba::new(23,23,23,255));

                self.renderer.render(ctx, self.state, &self.camera, &mut self.world, &mut self.old_world)?;
                self.text_renderer.render(ctx)?;
            },
            Event::Resize(width, height) => {
                assert!(width != 0 && height != 0);
                ctx.viewport(0, 0, width as i32, height as i32);
                self.camera.parent.aspect = width as f32 / height as f32;
                self.world.resize(ctx, width, height)?;
                self.old_world.resize(ctx, width, height)?;
                camera_update = true;
            },
            Event::Click(pos) => match self.state{
                GameState::InProgress => {
                    let pt = self.camera.to_world_coords(pos);
                    self.world.try_rotate(pt.into());
                    if self.world.is_completed() {
                        self.state.set(GameState::WaitingForEnd(pt));
                    }
                }
                GameState::Ending(_, _) => {
                    self.state.set(GameState::Ended);
                    camera_update = true;
                },
                GameState::Ended => {
                    let pt = self.camera.to_world_coords(pos);
                    let mut new_world = World::new(self.world.seed() + 1);
                    new_world.scramble();
                    std::mem::swap(&mut self.world, &mut self.old_world);
                    self.world.reinitialize(new_world);
                    self.state.set(GameState::Transition(pt, 0.0));
                    self.text_renderer.set_text(&format!("Level {}", self.world.seed()));
                    camera_update = true;
                },
                GameState::Transition(_, _) => {
                    self.state.set(GameState::InProgress);
                    camera_update = true;
                }
                _ => {}
            },
            Event::Zoom(center, amount, animate) => {
                self.camera.zoom(center, amount, animate);
                camera_update = true;
            }
            Event::Drag(delta) => {
                self.camera.move_by(self.camera.to_world_coords(-delta) - self.camera.to_world_coords(Vec2::ZERO));
                camera_update = true;
            }
            Event::TouchStart => self.camera.capture(),
            Event::TouchEnd => self.camera.release()
        }
        Ok(camera_update || self.camera.update_required() || self.world.update_required() || self.state.is_animated())
    }
}

