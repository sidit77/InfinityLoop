mod meshes;

use glam::{Mat4, Vec2};
use miniquad::*;

struct Game {
    pipeline: Pipeline,
    bindings: Bindings,
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

        Game { pipeline, bindings }
    }
}

impl EventHandler for Game {
    fn update(&mut self, _ctx: &mut Context) {}

    fn draw(&mut self, ctx: &mut Context) {
        let t = date::now();

        ctx.begin_default_pass(Default::default());

        ctx.apply_pipeline(&self.pipeline);
        ctx.apply_bindings(&self.bindings);

        ctx.apply_uniforms(&shader::Uniforms {
            camera: Mat4::IDENTITY,
            model: Mat4::IDENTITY,
            color: [1.0,0.0,0.0,1.0],
            click_pos: Default::default(),
            radius: 0.0
        });
        let model = meshes::MODEL2;
        ctx.draw(model.start, model.len() as i32, 1);
        ctx.end_render_pass();

        ctx.commit_frame();
    }
}

fn main() {
    miniquad::start(conf::Conf::default(), |mut ctx| {
        UserData::owning(Game::new(&mut ctx), ctx)
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