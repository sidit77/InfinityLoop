#![windows_subsystem = "windows"]

mod opengl;
mod types;
mod meshes;
mod app;
mod camera;
mod world;
mod util;

use std::ops::Sub;
use std::time::Duration;
use glam::{Mat4, Quat, Vec2};
use crate::app::{Event, EventHandler};
use crate::camera::Camera;
use crate::opengl::{Buffer, BufferTarget, Context, DataType, PrimitiveType, SetUniform, Shader, ShaderProgram, ShaderType, VertexArray, VertexArrayAttribute};
use crate::types::{Color, HexPos};
use crate::world::World;

struct Game {
    _vertex_buffer: Buffer,
    _index_buffer: Buffer,
    vertex_array: VertexArray,
    program: ShaderProgram,
    camera: Camera,
    world: World
}

impl Game {

    fn new(ctx: &Context) -> Self {
        let vertex_array = VertexArray::new(&ctx).unwrap();
        ctx.use_vertex_array(&vertex_array);

        let vertex_buffer = Buffer::new(&ctx, BufferTarget::Array).unwrap();
        vertex_buffer.set_data::<f32>(&[
            -1., -1., 0., 0.,
             1., -1., 1., 0.,
             1.,  1., 1., 1.,
            -1.,  1., 0., 1.,
        ]);

        let index_buffer = Buffer::new(&ctx, BufferTarget::ElementArray).unwrap();
        index_buffer.set_data::<u16>(&[
            0, 1, 2,
            0, 2, 3
        ]);

        vertex_array.set_bindings(&[
            VertexArrayAttribute::Float(DataType::F32, 2, false),
            VertexArrayAttribute::Float(DataType::F32, 2, false)
        ]);

        let program = ShaderProgram::new(&ctx, &[
            &Shader::new(&ctx, ShaderType::Vertex, include_str!("shader/vertex.glsl")).unwrap(),
            &Shader::new(&ctx, ShaderType::Fragment, include_str!("shader/fragment.glsl")).unwrap(),
        ]).unwrap();

        let mut camera = Camera::default();

        camera.scale = 6.0;

        let mut world = World::new(1337);
        world.scramble();

        Self {
            vertex_array,
            _vertex_buffer: vertex_buffer,
            _index_buffer: index_buffer,
            program,
            camera,
            world
        }
    }
}

impl EventHandler for Game {
    fn draw(&mut self, ctx: &Context, _delta: Duration) {

        ctx.clear(Color::new(46, 52, 64, 255));

        ctx.use_vertex_array(&self.vertex_array);
        ctx.use_program(&self.program);

        self.program.set_uniform_by_name("camera", self.camera.to_matrix());

        //self.program.set_uniform_by_name("color", Color::new(200, 200, 200, 255));
        self.program.set_uniform_by_name("model", Mat4::IDENTITY);
        ctx.draw_elements_range(PrimitiveType::Triangles, DataType::U16, 0..6);

        //let tile = TileType::Tile0134;
        //let pos = HexPos::CENTER;
//
        //self.program.set_uniform_by_name("model", Mat4::from_translation(Vec2::from(pos).extend(0.0)));
        //self.program.set_uniform_by_name("color", Color::new(255, 100, 100, 255));
        //ctx.draw_elements_range(PrimitiveType::Triangles, DataType::U16, meshes::HEXAGON);
        //self.program.set_uniform_by_name("color", Color::new(100, 255, 100, 255));
        //ctx.draw_elements_range(PrimitiveType::Triangles, DataType::U16, tile.model());
//
        //for (i, n) in pos.neighbors().enumerate(){
        //    if tile.endings()[i] {
        //        self.program.set_uniform_by_name("model", Mat4::from_translation(Vec2::from(n).extend(0.0)));
        //        self.program.set_uniform_by_name("color", Color::new(100, 100, 255, 255));
        //        ctx.draw_elements_range(PrimitiveType::Triangles, DataType::U16, meshes::HEXAGON);
        //    }
        //}

        //for (hex, conf) in self.world.iter() {
        //    if !conf.is_empty() {
        //        self.program.set_uniform_by_name("color", Color::new(200, 200, 200, 255));
        //        self.program.set_uniform_by_name("model", Mat4::from_rotation_translation(
        //            Quat::from_rotation_z(conf.angle().to_radians()),
        //            Vec2::from(hex).extend(0.0)));
        //        ctx.draw_elements_range(PrimitiveType::Triangles, DataType::U16, conf.model());
        //    }
        //}



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
                self.world.try_rotate(pt);
                if self.world.is_completed() {
                    self.world = World::new(self.world.seed() + 1);
                    self.world.scramble();
                }
                //for pt in HexPos::spiral_iter(pt, 3) {
                //    if !self.hexagons.remove(&pt) {
                //        self.hexagons.insert(pt);
                //    }
                //}
                //self.world = World::new(self.world.seed() + 1);

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
