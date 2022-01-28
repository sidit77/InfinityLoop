mod opengl;
mod types;
mod meshes;
mod app;
mod camera;
mod world;
//mod intersection;

use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::hash::{Hash, Hasher};
use std::ops::Sub;
use std::time::Duration;
use glam::{Mat4, Vec2};
use crate::app::{Event, EventHandler};
use crate::camera::Camera;
use crate::opengl::{Buffer, BufferTarget, Context, DataType, PrimitiveType, SetUniform, Shader, ShaderProgram, ShaderType, VertexArray, VertexArrayAttribute};
use crate::types::{Color, HexPosition};

struct Game {
    _vertex_buffer: Buffer,
    _index_buffer: Buffer,
    vertex_array: VertexArray,
    program: ShaderProgram,
    camera: Camera,
    hexagons: HashSet<HexPosition>
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

        let mut camera = Camera::default();

        camera.scale = 6.0;

        let mut hexagons = HashSet::new();
        hexagons.insert(HexPosition::new(0,0));
        hexagons.insert(HexPosition::new(1,0));
        hexagons.insert(HexPosition::new(0,1));

        Self {
            vertex_array,
            _vertex_buffer: vertex_buffer,
            _index_buffer: index_buffer,
            program,
            camera,
            hexagons
        }
    }
}

impl EventHandler for Game {
    fn draw(&mut self, ctx: &Context, _delta: Duration) {

        ctx.clear(Color::new(46, 52, 64, 255));

        ctx.use_vertex_array(&self.vertex_array);
        ctx.use_program(&self.program);

        self.program.set_uniform_by_name("camera", self.camera.to_matrix());

        for hex in &self.hexagons {
            let mut hasher = DefaultHasher::new();
            hex.hash(&mut hasher);
            let rng = fastrand::Rng::with_seed(hasher.finish());
            self.program.set_uniform_by_name("color", Color::new(rng.u8(30..), rng.u8(30..), rng.u8(30..), 255));
            self.program.set_uniform_by_name("model", Mat4::from_translation(Vec2::from(*hex).extend(0.0)));
            ctx.draw_elements_range(PrimitiveType::Triangles, DataType::U16, meshes::HEXAGON);
        }



        //for i in self.world.indices() {
        //    let position = self.world.get_position(i);
        //    if let WorldElement::Tile(id, rotation) = self.world.get_element(i) {
        //        let tile_config = TileConfig::from(*id);
//
        //        let model = Mat4::from_rotation_translation(
        //            Quat::from_rotation_z(rotation.to_radians()),
        //            position.extend(0.0),
        //        );
//
        //        *rotation = Angle::lerp(*rotation, tile_config.angle(), 1.0 - f32::exp(-20.0 * delta.as_secs_f32()));
//
        //        self.program.set_uniform_by_name("model", model);
//
        //        ctx.draw_elements_range(PrimitiveType::Triangles, DataType::U16, tile_config.model());
//
        //    }
        //}

    }

    fn event(&mut self, event: app::Event) {
        match event {
            Event::WindowResize(width, height) => {
                self.camera.aspect = width / height;
            }
            Event::Click(pos) => {
                let pt = self.camera.to_world_coords(pos).into();
                for pt in HexPosition::spiral_iter(pt, 3) {
                    if !self.hexagons.remove(&pt) {
                        self.hexagons.insert(pt);
                    }
                }

            },
            Event::Zoom(center, amount) => {
                let camera = &mut self.camera;
                let old = camera.to_world_coords(center);
                camera.scale = camera.scale.sub(amount * (camera.scale / 10.0)).max(1.0);
                let new = camera.to_world_coords(center);
                camera.position += old - new;
            }
            Event::Drag(delta) => {
                self.camera.position += self.camera.to_world_coords(-delta.absolute()) - self.camera.to_world_coords(Vec2::ZERO);
            },
            Event::DragEnd(_) => {},
            Event::DragStart(_) => {}
        }
    }
}


fn main() {
    app::run(|ctx| Game::new(ctx))
}
