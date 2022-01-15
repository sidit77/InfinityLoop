#[cfg_attr(target_arch = "wasm32", path="platform/wasm.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path="platform/glutin.rs")]
mod platform;
mod opengl;
mod types;
mod meshes;

use log::{info, Level};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use crate::opengl::{Buffer, BufferTarget, DataType, PrimitiveType, Shader, ShaderProgram, ShaderType, VertexArray, VertexArrayAttribute};
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

    let vertex_buffer = Buffer::new(&ctx, BufferTarget::Array).unwrap();
    vertex_buffer.set_data(meshes::VERTICES);

    let index_buffer = Buffer::new(&ctx, BufferTarget::ElementArray).unwrap();
    index_buffer.set_data(meshes::INDICES);

    vertex_array.set_bindings(&[VertexArrayAttribute::Float(DataType::F32, 2, false)]);


    let (vertex_shader_source, fragment_shader_source) = (
        r#"#version 300 es
        layout(location = 0) in vec2 pos;
        out vec2 vert;
        void main() {
            vert = pos * 0.3;
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
                ctx.draw_elements_range(PrimitiveType::Triangles, DataType::U16, meshes::MODEL7);
                window.swap_buffers().unwrap();
            }
            Event::WindowEvent { ref event, .. } => match event {
                WindowEvent::Resized(physical_size) => {
                    window.resize(*physical_size);
                    ctx.viewport(0, 0, physical_size.width as i32, physical_size.height as i32);
                    info!("{:?}", physical_size)
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
