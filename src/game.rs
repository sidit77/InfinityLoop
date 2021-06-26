use crate::camera::Camera;
use crate::intersection::Hexagon;
use crate::{meshes, SaveManager, vibrate_once};
use crate::shader::compile_program;
use crate::world::{TileConfig, World, WorldElement, WorldSave};
use css_color_parser::Color;
use glam::{Mat4, Quat, Vec2, Vec3, Vec3Swizzles};
use std::ops::Range;
use web_sys::{WebGl2RenderingContext, WebGlUniformLocation};
use std::time::Duration;
use std::collections::HashMap;

pub enum GameEvent<'a> {
    Resize(u32, u32),
    Click(f32, f32),
    SolveButton,
    ScrambleButton,
    SaveReceived(&'a str),
    Quitting
}

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

pub struct Game {
    gl: WebGl2RenderingContext,
    color: Color,
    camera: Camera,
    uniforms: HashMap<&'static str, WebGlUniformLocation>,
    world: World,
    rng: fastrand::Rng,
    state: GameState,
    save_manager: SaveManager
}

impl Game {
    pub fn new(
        gl: WebGl2RenderingContext,
        color: Color,
    ) -> Result<Self, String> {
        let program = compile_program(
            &gl,
            &[
                (
                    WebGl2RenderingContext::VERTEX_SHADER,
                    include_str!("shader/vertex.glsl"),
                ),
                (
                    WebGl2RenderingContext::FRAGMENT_SHADER,
                    include_str!("shader/fragment.glsl"),
                ),
            ],
        )?;
        gl.use_program(Some(&program));

        let buffer = gl.create_buffer().ok_or("failed to create buffer")?;
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        gl.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            meshes::VERTEX_DATA,
            WebGl2RenderingContext::STATIC_DRAW,
        );
        gl.vertex_attrib_pointer_with_i32(0, 2, WebGl2RenderingContext::FLOAT, false, 0, 0);
        gl.enable_vertex_attrib_array(0);

        gl.clear_color(0.0, 0.0, 0.0, 0.0);

        let rng = fastrand::Rng::with_seed(1337);
        //console_log!("{:?}", crate::renderer::meshes::MODEL1);

        let mut save_manager = SaveManager::new();

        let world = match save_manager.load_world() {
            None => {
                let mut world = World::from_seed(1);
                world.scramble(&rng);
                world
            }
            Some(save) => save.into(),
        };

        let camera = Camera {
            position: Vec2::new(0.0, 1.0),
            ..Camera::default()
        };

        let mut uniforms = HashMap::new();
        for u in &["camera", "model", "color", "clickPos", "radius"] {
            if let Some(loc) = gl.get_uniform_location(&program, *u){
                uniforms.insert(*u, loc);
            }
        }

        let state = GameState::from_world(&world);

        Ok(Self {
            gl,
            color,
            camera,
            uniforms,
            world,
            rng,
            state,
            save_manager
        })
    }

    pub fn on_event(&mut self, event: GameEvent){
        match event {
            GameEvent::Resize(width, height) => {
                self.camera.calc_aspect(width, height);
                self.camera.scale = {
                    let (w, h) = self.world.get_size();
                    f32::max((w / self.camera.aspect) * 0.62, h * 0.6)
                };

                self.gl.viewport(0, 0, width as i32, height as i32);
                self.gl.uniform2f(self.uniforms.get("screenSize"), width as f32, height as f32);
                self.gl.uniform1f(self.uniforms.get("aspect"), self.camera.aspect);

                self.gl.uniform_matrix4fv_with_f32_array(
                    self.uniforms.get("camera"), false, &self.camera.to_matrix().to_cols_array(),
                );

                //self.gl.uniform_matrix4fv_with_f32_array(Some(&self.mvp_location), false, &self.camera.to_matrix().to_cols_array());
            }
            GameEvent::Click(x, y) => match self.state {
                GameState::InProgress => {
                    let point = Vec3::new(2.0 * x - 1.0, 2.0 * (1.0 - y) - 1.0, 0.0);
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

                                vibrate_once();
                            }
                        }
                    }
                    if self.world.is_completed() {
                        self.state = GameState::Ending(point.xy(), 0.0);
                    }
                }
                GameState::Ended => {
                    self.world = World::from_seed(self.world.seed() + 1);
                    self.world.scramble(&self.rng);
                    self.state = GameState::from_world(&self.world);
                }
                _ => {}
            }
            GameEvent::SolveButton => {
                self.world = World::from_seed(self.world.seed());
                if let WorldElement::Tile(index, _) = self.world.get_element(0) {
                    *index = TileConfig::from(*index).rotate_by(1).index();
                }
                self.state = GameState::from_world(&self.world);
            }
            GameEvent::ScrambleButton => {
                self.world.scramble(&self.rng);
                self.state = GameState::from_world(&self.world);
            }
            GameEvent::SaveReceived(save_str) => {
                //console_log!("R: {}", save_str);
                if let Some(ws) = self.save_manager.handle_world_update(save_str){
                    console_log!("Reloading...");
                    self.world = ws.into();
                    self.state = GameState::from_world(&self.world);
                }
            }
            GameEvent::Quitting => {
                self.save_manager.save_world(&WorldSave::from(&self.world));
                self.save_manager.flush();
            }
        }
    }

    pub fn render(&mut self, time: Duration) {
        {
            let fc = self.color.as_f32();
            self.gl.uniform4f(self.uniforms.get("color"), fc[0], fc[1], fc[2], fc[3]);
            self.gl.uniform1f(self.uniforms.get("radius"), self.state.get_anim_radius());
            if let GameState::Ending(p, r) = self.state {
                self.gl.uniform2f(self.uniforms.get("clickPos"), p.x as f32, p.y as f32);
                self.state = match r > self.camera.scale + (self.camera.position - p).length() {
                    true => GameState::Ended,
                    false => GameState::Ending(p, r + 12.0 * time.as_secs_f32())
                }
            }
        }
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        //let rng = fastrand::Rng::with_seed(1337);

        for i in self.world.indices() {
            let position = self.world.get_position(i);
            if let WorldElement::Tile(id, rotation) = self.world.get_element(i) {
                let tile_config = TileConfig::from(*id);

                let obj_mat = Mat4::from_rotation_translation(
                        Quat::from_rotation_z(*rotation),
                        position.extend(0.0),
                    );

                *rotation = lerp_radians(
                    *rotation,
                    tile_config.radian_rotation(), 1.0 - f32::exp(-20.0 * time.as_secs_f32()));

                self.gl.uniform_matrix4fv_with_f32_array(
                    self.uniforms.get("model"),
                    false,
                    &obj_mat.to_cols_array(),
                );
                //self.gl.uniform4f(Some(&self.color_location), rng.f32(), rng.f32(), rng.f32(), 1.0);
                //self.gl.draw_array_range(WebGl2RenderingContext::TRIANGLES, meshes::HEXAGON);
                //self.gl.uniform4f(Some(&self.color_location), 0.0, 0.0, 0.0, 1.0);
                self.gl
                    .draw_array_range(WebGl2RenderingContext::TRIANGLES, tile_config.model());
            }
        }
    }

    pub fn world(&self) -> &World {
        &self.world
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

trait DrawRange {
    fn draw_array_range(&self, mode: u32, range: std::ops::Range<i32>);
}

impl DrawRange for WebGl2RenderingContext {
    fn draw_array_range(&self, mode: u32, range: Range<i32>) {
        self.draw_arrays(mode, range.start, range.len() as i32);
    }
}

trait AsF32 {
    fn as_f32(&self) -> [f32; 4];
}

impl AsF32 for Color {
    fn as_f32(&self) -> [f32; 4] {
        [
            self.r as f32 / u8::MAX as f32,
            self.g as f32 / u8::MAX as f32,
            self.b as f32 / u8::MAX as f32,
            self.a,
        ]
    }
}
