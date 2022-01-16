mod opengl;
mod types;
mod meshes;
mod app;
mod camera;
mod world;
mod intersection;

use std::ops::Sub;
use std::time::Duration;
use fastrand::Rng;
use glam::{Mat4, Quat, Vec2};
use crate::app::{Event, EventHandler};
use crate::camera::Camera;
use crate::intersection::Hexagon;
use crate::opengl::{Buffer, BufferTarget, Context, DataType, PrimitiveType, SetUniform, Shader, ShaderProgram, ShaderType, VertexArray, VertexArrayAttribute};
use crate::types::{Angle, Color};
use crate::world::{TileConfig, World, WorldElement};

#[derive(Debug, Copy, Clone, PartialEq)]
enum GameState {
    InProgress,
    Ending(Vec2, f32),
    Ended
}

impl GameState {
    fn from_world(world: &World) -> Self {
        match world.is_completed() {
            true => Self::Ended,
            false => Self::InProgress
        }
    }

    fn get_anim_radius(self) -> f32{
        match self {
            GameState::InProgress => 0.0,
            GameState::Ending(_, r) => r,
            GameState::Ended => f32::INFINITY
        }
    }

    fn get_click_pos(self) -> Vec2 {
        match self {
            GameState::Ending(pos, _) => pos,
            _ => Vec2::ZERO
        }
    }

}


struct Game {
    _vertex_buffer: Buffer,
    _index_buffer: Buffer,
    vertex_array: VertexArray,
    program: ShaderProgram,
    camera: Camera,
    world: World,
    rng: Rng,
    state: GameState,
    camera_vel: Vec2
}

impl Game {

    fn new(ctx: &Context) -> Self {
        let vertex_array = VertexArray::new(&ctx).unwrap();
        ctx.use_vertex_array(&vertex_array);

        let vertex_buffer = Buffer::new(&ctx, BufferTarget::Array).unwrap();
        vertex_buffer.set_data(meshes::VERTICES);

        let index_buffer = Buffer::new(&ctx, BufferTarget::ElementArray).unwrap();
        index_buffer.set_data(meshes::INDICES);

        vertex_array.set_bindings(&[VertexArrayAttribute::Float(DataType::F32, 2, false)]);

        let program = ShaderProgram::new(&ctx, &[
            &Shader::new(&ctx, ShaderType::Vertex, include_str!("shader/vertex.glsl")).unwrap(),
            &Shader::new(&ctx, ShaderType::Fragment, include_str!("shader/fragment.glsl")).unwrap(),
        ]).unwrap();

        let rng = fastrand::Rng::with_seed(1337);
        let mut world = World::from_seed(1);
        world.scramble(&rng);

        let state = GameState::from_world(&world);

        let mut camera = Camera::default();

        camera.scale = 6.0;

        Self {
            vertex_array,
            _vertex_buffer: vertex_buffer,
            _index_buffer: index_buffer,
            program,
            camera,
            world,
            rng,
            state,
            camera_vel: Vec2::ZERO
        }
    }
}

impl EventHandler for Game {
    fn draw(&mut self, ctx: &Context, delta: Duration) {

        self.camera.position += self.camera_vel;
        let dt = 1.0;
        self.camera_vel *= 1.0 / (1.0 + dt * 0.02);

        if let GameState::Ending(p, r) = self.state {
            self.state = match r > self.camera.scale + (self.camera.position - p).length() {
                true => GameState::Ended,
                false => GameState::Ending(p, r + 12.0 * delta.as_secs_f32())
            }
        }

        ctx.clear(Color::new(46, 52, 64, 255));

        ctx.use_vertex_array(&self.vertex_array);
        ctx.use_program(&self.program);

        self.program.set_uniform_by_name("camera", self.camera.to_matrix());
        self.program.set_uniform_by_name("color", Color::new(76, 86, 106, 255));
        self.program.set_uniform_by_name("clickPos", self.state.get_click_pos());
        self.program.set_uniform_by_name("radius", self.state.get_anim_radius());


        for i in self.world.indices() {
            let position = self.world.get_position(i);
            if let WorldElement::Tile(id, rotation) = self.world.get_element(i) {
                let tile_config = TileConfig::from(*id);

                let model = Mat4::from_rotation_translation(
                    Quat::from_rotation_z(rotation.to_radians()),
                    position.extend(0.0),
                );

                *rotation = Angle::lerp(*rotation, tile_config.angle(), 1.0 - f32::exp(-20.0 * delta.as_secs_f32()));

                self.program.set_uniform_by_name("model", model);

                ctx.draw_elements_range(PrimitiveType::Triangles, DataType::U16, tile_config.model());

            }
        }

    }

    fn event(&mut self, event: app::Event) {
        match event {
            Event::WindowResize(width, height) => {
                self.camera.aspect = width / height;
            }
            Event::Click(pos) => match self.state {
                GameState::InProgress => {
                    let point = self.camera.to_world_coords(pos);

                    for i in self.world.indices() {
                        let position = self.world.get_position(i);
                        if let WorldElement::Tile(index, _) = self.world.get_element(i) {
                            let hex = Hexagon {
                                position,
                                rotation: 0.0,
                                radius: 1.0,
                            };
                            if hex.contains(point) {
                                *index = TileConfig::from(*index).rotate_by(1).index();
                            }
                        }
                    }
                    if self.world.is_completed() {
                        self.state = GameState::Ending(point, 0.0);
                    }
                },
                GameState::Ended => {
                    self.world = World::from_seed(self.world.seed() + 1);
                    self.world.scramble(&self.rng);
                    self.state = GameState::from_world(&self.world);
                    let bb = self.world.get_bounding_box();
                    //camera.rotation = Angle::degrees(90.0);
                    self.camera.position = bb.center();
                    self.camera.scale = f32::max((bb.height() / self.camera.aspect) * 0.51, bb.width() * 0.51);
                    //center_camera(&mut self.camera, &self.world);
                },
                _ => {}
            },
            Event::Zoom(center, amount) => {
                let camera = &mut self.camera;
                let old = camera.to_world_coords(center);
                camera.scale = camera.scale.sub(amount * (camera.scale / 10.0)).max(1.0);
                let new = camera.to_world_coords(center);
                camera.position += old - new;
            }
            Event::Drag(delta) => {
                self.camera.position += self.camera.to_world_coords(-delta) - self.camera.to_world_coords(Vec2::ZERO);
            },
            Event::DragEnd(delta) => {
                self.camera_vel = self.camera.to_world_coords(-delta) - self.camera.to_world_coords(Vec2::ZERO);
            },
            Event::Touch => {
                self.camera_vel = Vec2::ZERO;
            }
        }
    }
}


fn main() {
    app::run(|ctx| Game::new(ctx))
}
