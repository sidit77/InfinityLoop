use std::time::Duration;
use glam::Vec2;
use serde::{Serialize, Deserialize};
use crate::{AppContext, Camera, RenderableWorld};
use crate::opengl::*;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub enum GameState {
    Tutorial,
    Shuffeling,
    InProgress,
    WaitingForEnd(Vec2),
    Ending(Vec2, f32),
    Ended,
    Transition(Vec2, f32)
}

impl GameState {

    pub fn set(&mut self, new_state: GameState) {
        log::trace!("State transition: {:?} -> {:?}", self, new_state);
        *self = new_state;
    }

    pub fn update(&mut self, delta: Duration, world_updating: bool) {
        match *self {
            GameState::Shuffeling => if !world_updating {
                self.set(GameState::InProgress);
            }
            GameState::Ending(_, ref mut time) => {
                *time += delta.as_secs_f32();
                if *time > 1.5 {
                    self.set(GameState::Ended);
                }
            },
            GameState::WaitingForEnd(center) => if !world_updating {
                self.set(GameState::Ending(center, 0.0));
            },
            GameState::Transition(_, ref mut time) => {
                *time += delta.as_secs_f32();
                if *time > 1.5 {
                    self.set(GameState::InProgress);
                }
            },
            _ => {}
        }
    }

    pub fn is_animated(&self) -> bool {
        matches!(self,
            GameState::Shuffeling |
            GameState::Ending(_, _) |
            GameState::Transition(_, _) |
            GameState::WaitingForEnd(_))
    }

    pub fn update_speed(&self) -> u32 {
        match self {
            GameState::Shuffeling => 5,
            _ => 1
        }
    }

    pub fn is_interactive(&self) -> bool {
        !matches!(self, GameState::Shuffeling | GameState::Tutorial )
    }

}

pub struct GameRenderer {
    standard_shader: ShaderProgram,
    ending_shader: ShaderProgram,
    transition_shader: ShaderProgram
}

impl GameRenderer {

    pub fn new(ctx: &Context) -> GlResult<Self> {
        let vertex = Shader::new(ctx, ShaderType::Vertex, include_str!("../shader/postprocess.vert"))?;
        let standard_fragment = Shader::new(ctx, ShaderType::Fragment, include_str!("../shader/postprocess_standard.frag"))?;
        let ending_fragment = Shader::new(ctx, ShaderType::Fragment, include_str!("../shader/postprocess_ending.frag"))?;
        let transition_fragment = Shader::new(ctx, ShaderType::Fragment, include_str!("../shader/postprocess_transition.frag"))?;

        let standard_shader = ShaderProgram::new(ctx, &[&vertex, &standard_fragment])?;
        ctx.use_program(&standard_shader);
        ctx.set_uniform(&standard_shader.get_uniform("tex")?, 0);

        let ending_shader = ShaderProgram::new(ctx, &[&vertex, &ending_fragment])?;
        ctx.use_program(&ending_shader);
        ctx.set_uniform(&ending_shader.get_uniform("tex")?, 0);

        let transition_shader = ShaderProgram::new(ctx, &[&vertex, &transition_fragment])?;
        ctx.use_program(&transition_shader);
        ctx.set_uniform(&transition_shader.get_uniform("tex1")?, 0);
        ctx.set_uniform(&transition_shader.get_uniform("tex2")?, 1);

        Ok(Self {
            standard_shader,
            ending_shader,
            transition_shader
        })
    }

    pub fn render<A: AppContext>(&self, ctx: &A, state: GameState, camera: &Camera, world: &mut RenderableWorld, old_world: &mut RenderableWorld) -> GlResult<()> {
        world.render(ctx, camera);
        ctx.bind_texture(0, world.get_texture());
        match state {
            GameState::Tutorial | GameState::Shuffeling | GameState::InProgress | GameState::WaitingForEnd(_) => {
                ctx.use_program(&self.standard_shader);
                ctx.set_uniform(&self.standard_shader.get_uniform("completed")?, false);
            }
            GameState::Ended => {
                ctx.use_program(&self.standard_shader);
                ctx.set_uniform(&self.standard_shader.get_uniform("completed")?, true);
            }
            GameState::Ending(center, time) => {
                ctx.use_program(&self.ending_shader);
                ctx.set_uniform(&self.ending_shader.get_uniform("radius")?, f32::exp(2.0 * time) - 1.0); //
                ctx.set_uniform(&self.ending_shader.get_uniform("inv_camera")?, camera.to_matrix().inverse());
                ctx.set_uniform(&self.ending_shader.get_uniform("pxRange")?, ctx.screen_height() as f32 / (2.0 * camera.scale));
                ctx.set_uniform(&self.ending_shader.get_uniform("center")?, center);
            },
            GameState::Transition(center, time) => {
                old_world.render(ctx, camera);
                ctx.bind_texture(1, old_world.get_texture());
                ctx.use_program(&self.transition_shader);
                ctx.set_uniform(&self.transition_shader.get_uniform("radius")?, f32::exp(2.0 * time) - 1.0); //
                ctx.set_uniform(&self.transition_shader.get_uniform("inv_camera")?, camera.to_matrix().inverse());
                ctx.set_uniform(&self.transition_shader.get_uniform("pxRange")?, ctx.screen_height() as f32 / (2.0 * camera.scale));
                ctx.set_uniform(&self.transition_shader.get_uniform("center")?, center);
            }
        }
        ctx.use_framebuffer(None);
        ctx.set_blend_state(None);
        ctx.draw_arrays(PrimitiveType::TriangleStrip, 0, 4);
        Ok(())
    }

}