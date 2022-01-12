#[cfg_attr(target_arch = "wasm32", path="platform/wasm.rs")]
#[cfg_attr(not(target_arch = "wasm32"), path="platform/glutin.rs")]
mod platform;
mod opengl;
mod types;

use glow::HasContext;
use log::{info, Level};
use winit::dpi::PhysicalSize;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use crate::platform::WindowBuilderExt;
use crate::types::Color;


fn main() {
    platform::setup_logger(Level::Debug);

    let event_loop = EventLoop::new();
    let (window, gl) = WindowBuilder::new()
        .with_inner_size(PhysicalSize::new(1280, 720))
        .with_title("Infinity Loop")
        .build_context(&event_loop);
    
    unsafe {

        let vertex_array = gl
            .raw()
            .create_vertex_array()
            .expect("Cannot create vertex array");
        gl.raw().bind_vertex_array(Some(vertex_array));

        let program = gl.raw().create_program().expect("Cannot create program");

        let (vertex_shader_source, fragment_shader_source) = (
            r#"const vec2 verts[3] = vec2[3](
                vec2(0.5f, 1.0f),
                vec2(0.0f, 0.0f),
                vec2(1.0f, 0.0f)
            );
            out vec2 vert;
            void main() {
                vert = verts[gl_VertexID];
                gl_Position = vec4(vert - 0.5, 0.0, 1.0);
            }"#,
            r#"precision mediump float;
            in vec2 vert;
            out vec4 color;
            void main() {
                color = vec4(vert, 0.5, 1.0);
            }"#,
        );

        let shader_sources = [
            (glow::VERTEX_SHADER, vertex_shader_source),
            (glow::FRAGMENT_SHADER, fragment_shader_source),
        ];

        let mut shaders = Vec::with_capacity(shader_sources.len());

        for (shader_type, shader_source) in shader_sources.iter() {
            let shader = gl
                .raw()
                .create_shader(*shader_type)
                .expect("Cannot create shader");
            gl.raw().shader_source(shader, &format!("{}\n{}", "#version 300 es", shader_source));
            gl.raw().compile_shader(shader);
            if !gl.raw().get_shader_compile_status(shader) {
                panic!("{}", gl.raw().get_shader_info_log(shader));
            }
            gl.raw().attach_shader(program, shader);
            shaders.push(shader);
        }

        gl.raw().link_program(program);
        if !gl.raw().get_program_link_status(program) {
            panic!("{}", gl.raw().get_program_info_log(program));
        }

        for shader in shaders {
            gl.raw().detach_shader(program, shader);
            gl.raw().delete_shader(shader);
        }

        gl.raw().use_program(Some(program));

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
                    gl.clear(Color::new(46, 52, 64, 255));
                    gl.raw().draw_arrays(glow::TRIANGLES, 0, 3);
                    window.swap_buffers().unwrap();
                }
                Event::WindowEvent { ref event, .. } => match event {
                    WindowEvent::Resized(physical_size) => {
                        window.resize(*physical_size);
                        gl.raw().viewport(0, 0, physical_size.width as i32, physical_size.height as i32);
                        //error!("{:?}", physical_size)
                    }
                    WindowEvent::KeyboardInput { input, is_synthetic, .. } => info!("{:x} {:?} {}", input.scancode, input.virtual_keycode, is_synthetic),
                    WindowEvent::CloseRequested => {
                        gl.raw().delete_program(program);
                        gl.raw().delete_vertex_array(vertex_array);
                        *control_flow = ControlFlow::Exit
                    }
                    _ => (),
                },
                _ => (),
            }
        });
    }
}
