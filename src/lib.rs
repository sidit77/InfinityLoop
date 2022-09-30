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
use serde::{Serialize, Deserialize};

use crate::app::{AppContext, Event, Game};
use crate::camera::{AnimatedCamera, Camera};
use crate::types::{Color, HexPos, Rgba};
use crate::world::{World};
use crate::renderer::{Anchor, GameRenderer, GameState, RenderableWorld, TextAlignment, TextBuffer, TextRenderer, TileRenderResources};

pub mod export {
    pub use crate::opengl::Context;
    pub use crate::app::{GlowContext, Application, AppContext, Result};
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InfinityLoopBundle {
    world: World,
    camera: Camera,
    state: GameState
}

impl Default for InfinityLoopBundle {
    fn default() -> Self {
        let camera = Camera {
            scale: 6.0,
            ..Default::default()
        };

        let mut world = World::new(1);
        //world.scramble();

        //let state = match world.is_completed() {
        //    true => GameState::Ending(Vec2::ZERO, f32::INFINITY),
        //    false => GameState::Tutorial
        //};
        let state = GameState::Tutorial;

        Self {
            world,
            camera,
            state
        }
    }
}

pub struct InfinityLoop {
    renderer: GameRenderer,
    camera: AnimatedCamera,
    world: RenderableWorld,
    old_world: RenderableWorld,
    text_renderer: TextRenderer,
    text_buffer: TextBuffer,
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

        let text_renderer = TextRenderer::new(ctx, &ArteryFont::read(include_bytes!("font/arial.arfont").as_slice())?, (width, height))?;
        let mut text_buffer = text_renderer.create_buffer()?;
        match bundle.state {
            GameState::Tutorial | GameState::Shuffeling
              => text_buffer.set_text(&format!("Click the Screen to Start"), TextAlignment::Center),
            _ => text_buffer.set_text(&format!("Level {}", world.seed()), TextAlignment::Center),
        };
        text_buffer.anchor = Anchor::CenterTop;
        text_buffer.text_size = 60.0;
        text_buffer.offset = Vec2::new(0.0, -10.0);
        
        Ok(Self {
            renderer,
            camera,
            world,
            old_world,
            text_renderer,
            text_buffer,
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
                let old_state = self.state;
                self.state.update(delta, self.world.update_required());
                self.world.update(delta / self.state.update_speed());
                if matches!(self.state, GameState::Shuffeling) {
                    self.text_buffer.offset = Vec2::lerp(
                        self.text_buffer.offset,
                        self.text_buffer.offset + self.text_buffer.size() * Vec2::new(0.0, 1.0),
                        1.0 - f32::exp(-3.0 * delta.as_secs_f32()));
                }
                if matches!(old_state, GameState::Shuffeling) && matches!(self.state, GameState::InProgress){
                    self.text_buffer.set_text(&format!("Level {}", self.world.seed()), TextAlignment::Left);
                    self.text_buffer.offset = Vec2::new(0.0, -10.0);
                }

                ctx.clear(Rgba::new(23,23,23,255));

                self.renderer.render(ctx, self.state, &self.camera, &mut self.world, &mut self.old_world)?;
                self.text_renderer.render(ctx, &self.text_buffer)?;
            },
            Event::Resize(width, height) => {
                assert!(width != 0 && height != 0);
                ctx.viewport(0, 0, width as i32, height as i32);
                self.camera.parent.aspect = width as f32 / height as f32;
                self.world.resize(ctx, width, height)?;
                self.old_world.resize(ctx, width, height)?;
                self.text_renderer.resize(ctx, width, height)?;
                camera_update = true;
            },
            Event::Click(pos) => match self.state{
                GameState::Tutorial => {
                    //let pt = self.camera.to_world_coords(pos);
                    //if self.world.try_rotate(pt.into()) {
                    //    self.text_buffer.set_text(&format!("Level {}", self.world.seed()), TextAlignment::Left);
                    //    self.state.set(GameState::InProgress);
                    //}
                    //if self.world.is_completed() {
                    //    self.state.set(GameState::WaitingForEnd(pt));
                    //}
                    self.world.scramble();
                    self.state.set(GameState::Shuffeling);
                }
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
                    new_world.scramble(false);
                    std::mem::swap(&mut self.world, &mut self.old_world);
                    self.world.reinitialize(new_world);
                    self.state.set(GameState::Transition(pt, 0.0));
                    self.text_buffer.set_text(&format!("Level {}", self.world.seed()), TextAlignment::Left);
                    camera_update = true;
                },
                GameState::Transition(_, _) => {
                    self.state.set(GameState::InProgress);
                    camera_update = true;
                }
                _ => {}
            },
            Event::Zoom(center, amount, animate) => if self.state.is_interactive() {
                self.camera.zoom(center, amount, animate);
                camera_update = true;
            }
            Event::Drag(delta) => if self.state.is_interactive() {
                self.camera.move_by(self.camera.to_world_coords(-delta) - self.camera.to_world_coords(Vec2::ZERO));
                camera_update = true;
            }
            Event::TouchStart => if self.state.is_interactive() {
                self.camera.capture()
            },
            Event::TouchEnd => if self.state.is_interactive() {
                self.camera.release()
            }
        }
        Ok(camera_update || self.camera.update_required() || self.world.update_required() || self.state.is_animated())
    }
}
