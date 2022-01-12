#[cfg_attr(target_arch = "wasm32", path="platform/wasm.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path="platform/glutin.rs")]
mod platform;
mod opengl;
mod types;

use log::{info, Level};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use crate::opengl::{PrimitiveType, Shader, ShaderProgram, ShaderType, VertexArray};
use crate::platform::WindowBuilderExt;
use crate::types::Color;


fn main() {
    platform::setup_logger(Level::Debug);

    let event_loop = EventLoop::new();
    let (window, ctx) = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_title("Infinity Loop")
        .build_context(&event_loop);


    let vertex_array = VertexArray::new(&ctx).unwrap();
    ctx.use_vertex_array(&vertex_array);

    let (vertex_shader_source, fragment_shader_source) = (
        r#"#version 300 es
        const vec2 verts[3] = vec2[3](
            vec2(0.5f, 1.0f),
            vec2(0.0f, 0.0f),
            vec2(1.0f, 0.0f)
        );
        out vec2 vert;
        void main() {
            vert = verts[gl_VertexID];
            gl_Position = vec4(vert - 0.5, 0.0, 1.0);
        }"#,
        r#"#version 300 es
        precision mediump float;
        in vec2 vert;
        out vec4 color;
        void main() {
            color = vec4(vert, 0.5, 1.0);
        }"#,
    );

    let program = ShaderProgram::new(&ctx, &[
        &Shader::new(&ctx, ShaderType::Vertex, vertex_shader_source).unwrap(),
        &Shader::new(&ctx, ShaderType::Fragment, fragment_shader_source).unwrap(),
    ]).unwrap();

    ctx.use_program(&program);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        match event {
            Event::LoopDestroyed => {
                return;
            }
            Event::MainEventsCleared => {
                window.window().request_redraw();
            }
            Event::RedrawRequested(_) => {
                ctx.clear(Color::new(46, 52, 64, 255));
                ctx.draw(PrimitiveType::Triangles, 0, 3);
                window.swap_buffers().unwrap();
            }
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    window.resize(*physical_size);
                    ctx.viewport(0, 0, physical_size.width as i32, physical_size.height as i32);
                    //error!("{:?}", physical_size)
                }
                WindowEvent::KeyboardInput { input, is_synthetic, .. } => info!("{:x} {:?} {}", input.scancode, input.virtual_keycode, is_synthetic),
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit
                }
                _ => (),
            },
            _ => (),
        }
    });
}
