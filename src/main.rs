mod meshes;
mod camera;
mod intersection;
mod world;
mod events;
mod angle;

use std::ops::Sub;
use std::time::Duration;
use fastrand::Rng;
use glam::{Mat4, Quat, Vec2};
use miniquad::*;
use crate::angle::Angle;
use crate::camera::Camera;
use crate::events::{Event, EventHandlerMod, EventHandlerProxy};
use crate::intersection::Hexagon;
use crate::shader::Uniforms;
use crate::world::{TileConfig, World, WorldElement};

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

    fn get_anim_radius(&self) -> f32{
        match self {
            GameState::InProgress => 0.0,
            GameState::Ending(_, r) => *r,
            GameState::Ended => f32::INFINITY
        }
    }
}


struct Game {
    pipeline: Pipeline,
    bindings: Bindings,
    camera: Camera,
    world: World,
    rng: Rng,
    state: GameState,
    camera_vel: Vec2
}

impl Game {
    pub fn new(ctx: &mut Context) -> Game {

        let vertex_buffer = Buffer::immutable(ctx, BufferType::VertexBuffer, meshes::VERTICES);
        let index_buffer = Buffer::immutable(ctx, BufferType::IndexBuffer, meshes::INDICES);

        let bindings = Bindings {
            vertex_buffers: vec![vertex_buffer],
            index_buffer,
            images: Vec::new(),
        };

        let shader = Shader::new(ctx, shader::VERTEX, shader::FRAGMENT, shader::meta()).unwrap();

        let pipeline = Pipeline::new(
            ctx,
            &[BufferLayout::default()],
            &[VertexAttribute::new("pos", VertexFormat::Float2)],
            shader,
        );

        let rng = fastrand::Rng::with_seed(1337);
        let mut world = World::from_seed(1);
        world.scramble(&rng);

        let mut camera = Camera::default();
        let width = ctx.screen_size().0;
        let height = ctx.screen_size().1;
        camera.aspect = width / height;
        center_camera(&mut camera, &world);

        let state = GameState::from_world(&world);

        Game { pipeline, bindings, camera, world, rng, state, camera_vel: Default::default() }
    }
}

fn center_camera(camera: &mut Camera, world: &World){
    let bb = world.get_bounding_box();
    //camera.rotation = Angle::degrees(90.0);
    camera.position = bb.center();
    camera.scale = f32::max((bb.height() / camera.aspect) * 0.51, bb.width() * 0.51);
}

impl EventHandlerMod for Game {

    fn draw(&mut self, ctx: &mut Context, delta: Duration) {
        self.camera.position += self.camera_vel;
        self.camera_vel *= 0.98;
        let mut uniforms = Uniforms {
            camera: self.camera.to_matrix(),
            model: Mat4::IDENTITY,
            color: [0.30, 0.34, 0.42, 1.0],
            click_pos: Default::default(),
            radius: self.state.get_anim_radius()
        };
        if let GameState::Ending(p, r) = self.state {
            uniforms.click_pos = p;
            self.state = match r > self.camera.scale + (self.camera.position - p).length() {
                true => GameState::Ended,
                false => GameState::Ending(p, r + 12.0 * delta.as_secs_f32())
            }
        }


        ctx.begin_default_pass(Default::default());
        ctx.clear(Some((0.18, 0.20, 0.25, 1.0)), None, None);
        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);


        for i in self.world.indices() {
            let position = self.world.get_position(i);
            if let WorldElement::Tile(id, rotation) = self.world.get_element(i) {
                let tile_config = TileConfig::from(*id);

                uniforms.model = Mat4::from_rotation_translation(
                    Quat::from_rotation_z(rotation.to_radians()),
                    position.extend(0.0),
                );

                *rotation = Angle::lerp(*rotation, tile_config.angle(), 1.0 - f32::exp(-20.0 * delta.as_secs_f32()));

                ctx.apply_uniforms(&uniforms);

                let model = tile_config.model();
                ctx.draw(model.start, model.len() as i32, 1);

            }
        }

        ctx.end_render_pass();

        ctx.commit_frame();
    }

    fn event(&mut self, _ctx: &mut Context, event: Event) {
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
                    center_camera(&mut self.camera, &self.world);
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
                let ratio = Vec2::new(self.camera.aspect, 1.0) * (self.camera.scale / 15.0);
                self.camera.position += -delta * 30.0 * ratio;
            },
            Event::DragEnd(delta) => {
                let ratio = Vec2::new(self.camera.aspect, 1.0) * (self.camera.scale / 15.0);
                self.camera_vel = -delta * 30.0 * ratio;
            },
            Event::Touch => {
                self.camera_vel = Vec2::ZERO;
            }
        }
    }
}

fn main() {
    miniquad::start(conf::Conf::default(), |mut ctx| {
        UserData::owning(EventHandlerProxy::from(Game::new(&mut ctx)), ctx)
    });
}

mod shader {
    use glam::{Mat4, Vec2};
    use miniquad::*;

    pub const VERTEX: &str = include_str!("shader/vertex.glsl");
    pub const FRAGMENT: &str = include_str!("shader/fragment.glsl");

    pub fn meta() -> ShaderMeta {
        ShaderMeta {
            images: vec![],
            uniforms: UniformBlockLayout {
                uniforms: vec![
                    UniformDesc::new("camera", UniformType::Mat4),
                    UniformDesc::new("model", UniformType::Mat4),
                    UniformDesc::new("color", UniformType::Float4),
                    UniformDesc::new("clickPos", UniformType::Float2),
                    UniformDesc::new("radius", UniformType::Float1)
                ],
            },
        }
    }

    #[repr(C)]
    pub struct Uniforms {
        pub camera: Mat4,
        pub model: Mat4,
        pub color: [f32; 4],
        pub click_pos: Vec2,
        pub radius: f32
    }
}