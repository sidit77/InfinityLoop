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

pub enum GameEvent<'a> {
    Resize(u32, u32),
    Click(f32, f32),
    SolveButton,
    ScrambleButton,
    SaveReceived(&'a str),
    Quitting
}

pub struct Game {
    gl: WebGl2RenderingContext,
    color: Color,
    camera: Camera,
    mvp_location: WebGlUniformLocation,
    color_location: WebGlUniformLocation,
    world: World,
    rng: fastrand::Rng,
    finished: bool,
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

        let mvp_location = gl.get_uniform_location(&program, "mvp").unwrap();
        let color_location = gl.get_uniform_location(&program, "color").unwrap();

        let finished = world.is_completed();

        Ok(Self {
            gl,
            color,
            camera,
            mvp_location,
            color_location,
            world,
            rng,
            finished,
            save_manager
        })
    }

    pub fn on_event(&mut self, event: GameEvent){
        match event {
            GameEvent::Resize(width, height) => {
                self.gl.viewport(0, 0, width as i32, height as i32);

                self.camera.calc_aspect(width, height);
                self.camera.scale = {
                    let (w, h) = self.world.get_size();
                    f32::max((w / self.camera.aspect) * 0.62, h * 0.6)
                };
                //self.gl.uniform_matrix4fv_with_f32_array(Some(&self.mvp_location), false, &self.camera.to_matrix().to_cols_array());
            }
            GameEvent::Click(x, y) => {
                if self.finished {
                    self.world = World::from_seed(self.world.seed() + 1);
                    self.world.scramble(&self.rng);
                } else {
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
                }
            }
            GameEvent::SolveButton => {
                self.world = World::from_seed(self.world.seed());
            }
            GameEvent::ScrambleButton => {
                self.world.scramble(&self.rng);
            }
            GameEvent::SaveReceived(save_str) => {
                //console_log!("R: {}", save_str);
                if let Some(ws) = self.save_manager.handle_world_update(save_str){
                    console_log!("Reloading...");
                    self.world = ws.into();
                }
            }
            GameEvent::Quitting => {
                self.save_manager.save_world(&WorldSave::from(&self.world));
                self.save_manager.flush();
            }
        }
        {
            let finished = self.world.is_completed();
            if finished != self.finished {
                self.finished = finished;
                self.save_manager.save_world(&WorldSave::from(&self.world));
            }
        }
    }

    pub fn render(&mut self, time: Duration) {
        {
            let fc = self.color.as_f32();
            if self.finished {
                self.gl.uniform4f(
                    Some(&self.color_location),
                    1.0 - fc[0],
                    1.0 - fc[1],
                    1.0 - fc[2],
                    fc[3],
                );
            } else {
                self.gl.uniform4f(Some(&self.color_location), fc[0], fc[1], fc[2], fc[3]);
            }
        }
        self.gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        //let rng = fastrand::Rng::with_seed(1337);

        for i in self.world.indices() {
            let position = self.world.get_position(i);
            if let WorldElement::Tile(id, rotation) = self.world.get_element(i) {
                let tile_config = TileConfig::from(*id);

                let obj_mat = self.camera.to_matrix()
                    * Mat4::from_rotation_translation(
                        Quat::from_rotation_z(*rotation),
                        position.extend(0.0),
                    );

                *rotation = lerp_radians(
                    *rotation,
                    tile_config.radian_rotation(), 1.0 - f32::exp(-20.0 * time.as_secs_f32()));

                self.gl.uniform_matrix4fv_with_f32_array(
                    Some(&self.mvp_location),
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

    pub fn finished(&self) -> bool {
        self.finished
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
