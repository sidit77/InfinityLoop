mod meshes;
mod camera;
mod intersection;
mod world;
mod events;

use std::time::Duration;
use fastrand::Rng;
use glam::{Mat4, Quat, Vec2, Vec3, Vec3Swizzles};
use miniquad::*;
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
    state: GameState
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
        camera.calc_aspect(ctx.screen_size().0, ctx.screen_size().1);

        let state = GameState::from_world(&world);

        let mut g = Game { pipeline, bindings, camera, world, rng, state };
        g.center_camera();
        g
    }
}

impl Game {
    fn center_camera(&mut self){
        let bb = self.world.get_bounding_box();
        self.camera.rotation = std::f32::consts::FRAC_PI_2;
        self.camera.position = bb.center();
        self.camera.scale = {
            f32::max((bb.height() / self.camera.aspect) * 0.51, bb.width() * 0.51)
        };
    }
}

impl EventHandlerMod for Game {

    fn draw(&mut self, ctx: &mut Context, delta: Duration) {
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
                    Quat::from_rotation_z(*rotation),
                    position.extend(0.0),
                );

                *rotation = lerp_radians(
                    *rotation,
                    tile_config.radian_rotation(), 1.0 - f32::exp(-20.0 * delta.as_secs_f32()));

                ctx.apply_uniforms(&uniforms);

                let model = tile_config.model();
                ctx.draw(model.start, model.len() as i32, 1);

                //self.gl.uniform4f(Some(&self.color_location), rng.f32(), rng.f32(), rng.f32(), 1.0);
                //self.gl.draw_array_range(WebGl2RenderingContext::TRIANGLES, meshes::HEXAGON);
                //self.gl.uniform4f(Some(&self.color_location), 0.0, 0.0, 0.0, 1.0);
                //self.gl
                //    .draw_array_range(WebGl2RenderingContext::TRIANGLES, tile_config.model());
            }
        }

        ctx.end_render_pass();

        ctx.commit_frame();
    }

    fn event(&mut self, _ctx: &mut Context, event: Event) {
        match event {
            Event::WindowResize(width, height) => {
                self.camera.calc_aspect(width, height);
                self.center_camera();
            }
            Event::Click(x, y) => match self.state {
                GameState::InProgress => {
                    let point = Vec3::new(2.0 * x - 1.0, 2.0 * y - 1.0, 0.0);
                    let point = self.camera.to_matrix().inverse().transform_point3(point);

                    for i in self.world.indices() {
                        let position = self.world.get_position(i);
                        if let WorldElement::Tile(index, _) = self.world.get_element(i) {
                            let hex = Hexagon {
                                position,
                                rotation: 0.0,
                                radius: 1.0,
                            };
                            if hex.contains(point.xy()) {
                                *index = TileConfig::from(*index).rotate_by(1).index();

                            }
                        }
                    }
                    if self.world.is_completed() {
                        self.state = GameState::Ending(point.xy(), 0.0);
                    }
                },
                GameState::Ended => {
                    self.world = World::from_seed(self.world.seed() + 1);
                    self.world.scramble(&self.rng);
                    self.state = GameState::from_world(&self.world);
                    self.center_camera();
                },
                _ => {}
            }
        }
    }
}

fn lerp(a: f32, b: f32, lerp_factor: f32) -> f32{
    ((1.0 - lerp_factor) * a) + (lerp_factor * b)
}

fn lerp_radians(a: f32, mut b: f32, lerp_factor: f32) -> f32 {
    const PI: f32 = std::f32::consts::PI;
    const PI_TIMES_TWO: f32 = PI * 2.0;
    let diff = b - a;
    if diff < -PI {
        b += PI_TIMES_TWO;
        let result = lerp(a, b, lerp_factor);
        if result >= PI_TIMES_TWO {
            result - PI_TIMES_TWO
        } else {
            result
        }
    } else if diff > PI {
        b -= PI_TIMES_TWO;
        let result = lerp(a, b, lerp_factor);
        if result < 0.0 {
            result + PI_TIMES_TWO
        } else {
            result
        }
    } else {
        lerp(a, b, lerp_factor)
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