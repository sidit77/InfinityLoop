use std::time::Instant;
use glam::Vec2;
use glutin::{GlProfile};
use glutin::dpi::PhysicalSize;
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::{WindowBuilder};
use glutin::event::{ElementState, Event as WinitEvent, MouseButton, MouseScrollDelta, WindowEvent};
use infinity_loop::{Game, GlowContext, InfinityLoop};
use infinity_loop::export::{Context, Event as GameEvent, MouseDelta};

fn main() {
    run_andrid::<InfinityLoop>()
}

fn run_andrid<G: Game>() -> ! {
    let event_loop = EventLoop::new();
    let (window, ctx) = unsafe {
        let window_builder = WindowBuilder::new()
            .with_inner_size(PhysicalSize::new(1280, 720))
            .with_min_inner_size(PhysicalSize::new(100, 100))
            .with_title("Infinity Loop");
        let window = glutin::ContextBuilder::new()
            .with_vsync(true)
            .with_depth_buffer(0)
            .with_stencil_buffer(0)
            .with_gl_profile(GlProfile::Core)
            .build_windowed(window_builder, &event_loop)
            .expect("Can not get OpenGL context!")
            .make_current()
            .expect("Can set OpenGL context as current!");
        let gl = GlowContext::from_loader_function(|s| window.get_proc_address(s) as *const _);
        (window, gl)
    };
    let ctx = Context::from_glow(ctx);

    let mut handler = G::initialize(&ctx);


    let mut last_update = Instant::now();
    let mut mouse_tracker = MouseTracker::new();
    let mut dragging = false;

    event_loop.run(move |event, _, control_flow| match event {
        WinitEvent::WindowEvent { event, window_id, } if window_id == window.window().id() => match event {
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            WindowEvent::Resized(size) => {
                ctx.viewport(0, 0, size.width as i32, size.height as i32);
                handler.event(&ctx, GameEvent::WindowResize(size.width as f32, size.height as f32));
                window.resize(size);
            },
            WindowEvent::MouseWheel { delta, .. } => {
                let dy = match delta {
                    MouseScrollDelta::LineDelta(_, dy) => dy,
                    MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0
                };
                handler.event(&ctx, GameEvent::Zoom(mouse_tracker.position(), dy))
            },
            WindowEvent::CursorMoved {position, .. } => {
                let size = window.window().inner_size();
                mouse_tracker.update_position(Vec2::new(position.x as f32 / size.width as f32, 1.0 - position.y as f32 / size.height as f32));

                if dragging {
                    handler.event(&ctx, GameEvent::Drag(mouse_tracker.delta()))
                }
            }
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                handler.event(&ctx, GameEvent::Click(mouse_tracker.position()))
            },
            WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Middle, .. } => {
                dragging = true;
                handler.event(&ctx, GameEvent::DragStart(mouse_tracker.delta()))
            },
            WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Middle, .. } => {
                dragging = false;
                handler.event(&ctx, GameEvent::DragEnd(mouse_tracker.delta()))
            }
            _ => {}
        },
        WinitEvent::RedrawRequested(window_id) if window_id == window.window().id() => {
            let now = Instant::now();
            handler.draw(&ctx, now - last_update);
            last_update = now;
            *control_flow = ControlFlow::Poll;
            window.swap_buffers().expect("could not swap buffers");
        },
        WinitEvent::MainEventsCleared => window.window().request_redraw(),
        WinitEvent::LoopDestroyed => println!("Destroyed"),
        WinitEvent::Resumed => println!("Resumend"),
        WinitEvent::Suspended => println!("Suspended"),

        _ => {}
    });
}

struct MouseTracker {
    current_position: (Vec2, Instant),
    previous_position: (Vec2, Instant)
}

impl MouseTracker {

    fn new() -> Self {
        Self {
            current_position: (Vec2::ZERO, Instant::now()),
            previous_position: (Vec2::ZERO, Instant::now())
        }
    }

    fn position(&self) -> Vec2 {
        self.current_position.0
    }

    fn delta(&self) -> MouseDelta {
        MouseDelta(
            self.current_position.0 - self.previous_position.0,
            self.current_position.1.saturating_duration_since(self.previous_position.1)
        )
    }

    fn update_position(&mut self, position: Vec2) {
        self.previous_position = self.current_position;
        self.current_position = (position, Instant::now());
    }

}