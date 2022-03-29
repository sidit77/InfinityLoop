use std::time::Instant;
use android_logger::Config;
use glam::Vec2;
use glutin::{Api, ContextWrapper, GlRequest, PossiblyCurrent};
use glutin::event::{ElementState, Event, MouseButton, MouseScrollDelta, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};
use glutin::window::{Window, WindowBuilder};
use log::{info, Level};
use infinity_loop::{Game, GlowContext, InfinityLoop};
use infinity_loop::export::{Context, Event as GameEvent, MouseDelta};

#[cfg_attr(target_os = "android", ndk_glue::main(backtrace = "on"))]
fn main() {
    android_logger::init_once(
        Config::default()
            .with_min_level(Level::Trace)
            .with_tag("infinity_loop")
    );

    run_andrid::<InfinityLoop>()
}

struct Device<G: Game> {
    game: G,
    ctx: Context,
    window: ContextWrapper<PossiblyCurrent, Window>,
}

impl<G: Game> Device<G> {

    fn new(event_loop: &EventLoopWindowTarget<()>) -> Self {
        let (window, ctx) = unsafe {
            let window_builder = WindowBuilder::new()
                .with_title("Infinity Loop");
            let window = glutin::ContextBuilder::new()
                .with_vsync(true)
                .with_gl(GlRequest::Specific(Api::OpenGlEs, (3, 0)))
                .build_windowed(window_builder, &event_loop)
                .expect("Can not get OpenGL context!")
                .make_current()
                .expect("Can set OpenGL context as current!");
            let gl = GlowContext::from_loader_function(|s| window.get_proc_address(s) as *const _);
            (window, gl)
        };
        let ctx = Context::from_glow(ctx);
        let game = G::initialize(&ctx);
        Self {
            window,
            ctx,
            game
        }
    }

}

fn run_andrid<G: Game>() -> ! {
    info!("Starting the game");
    let event_loop = EventLoop::new();

    let mut handler: Option<Device<G>> = None;
    
    let mut last_update = Instant::now();
    let mut mouse_tracker = MouseTracker::new();
    let mut dragging = false;

    event_loop.run(move |event, _event_loop, control_flow| {
        if !matches!(event, Event::NewEvents(_) | Event::MainEventsCleared | Event::RedrawEventsCleared | Event::RedrawRequested(_)){
            info!("{:?}", event);
        }

        match event {
            Event::WindowEvent { event, window_id, } => if let Some(device) = handler.as_mut() {
                if window_id == device.window.window().id() {
                    match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(size) => {
                            device.ctx.viewport(0, 0, size.width as i32, size.height as i32);
                            device.game.event(&device.ctx, GameEvent::WindowResize(size.width as f32, size.height as f32));
                            device.window.resize(size);
                        },
                        WindowEvent::MouseWheel { delta, .. } => {
                            let dy = match delta {
                                MouseScrollDelta::LineDelta(_, dy) => dy,
                                MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0
                            };
                            device.game.event(&device.ctx, GameEvent::Zoom(mouse_tracker.position(), dy));
                        },
                        WindowEvent::CursorMoved { position, .. } => {
                            let size = device.window.window().inner_size();
                            mouse_tracker.update_position(Vec2::new(position.x as f32 / size.width as f32, 1.0 - position.y as f32 / size.height as f32));

                            if dragging {
                                device.game.event(&device.ctx, GameEvent::Drag(mouse_tracker.delta()));
                            }
                        }
                        WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Left, .. } => {
                            device.game.event(&device.ctx, GameEvent::Click(mouse_tracker.position()));
                        },
                        WindowEvent::MouseInput { state: ElementState::Pressed, button: MouseButton::Middle, .. } => {
                            dragging = true;
                            device.game.event(&device.ctx, GameEvent::DragStart(mouse_tracker.delta()));
                        },
                        WindowEvent::MouseInput { state: ElementState::Released, button: MouseButton::Middle, .. } => {
                            dragging = false;
                            device.game.event(&device.ctx, GameEvent::DragEnd(mouse_tracker.delta()));
                        },
                        _ => {}
                    }
                }
            },
            Event::RedrawRequested(window_id) => if let Some(device) = handler.as_mut() {
                if window_id == device.window.window().id() {
                    let now = Instant::now();
                    device.game.draw(&device.ctx, now - last_update);
                    last_update = now;
                    *control_flow = ControlFlow::Poll;
                    if device.window.swap_buffers().is_err() {
                        info!("Corrupted render context, try recovering ...");
                        if ndk_glue::native_window().is_some() {
                            *device = Device::new(_event_loop);
                            info!("... recovering successful!");
                        }
                    }
                }
            },
            Event::MainEventsCleared => if let Some(device) = handler.as_mut() {
                device.window.window().request_redraw()
            },
            Event::LoopDestroyed => println!("Destroyed"),
            Event::Resumed => {
                info!("Resumend");
                //enable_immersive();
                if handler.is_none() && ndk_glue::native_window().is_some() {
                    info!("trying to create screen");
                    handler = Some(Device::<G>::new(_event_loop));
                }

            },
            Event::Suspended => {
                info!("Suspended");
                handler = None;
            },

            _ => {}
        }
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

//fn enable_immersive() {
//    let vm_ptr = ndk_glue::native_activity().vm();
//    let vm = unsafe { jni::JavaVM::from_raw(vm_ptr) }.unwrap();
//    let env = vm.attach_current_thread_permanently().unwrap();
//    let activity = ndk_glue::native_activity().activity();
//    let window = env
//        .call_method(activity, "getWindow", "()Landroid/view/Window;", &[])
//        .unwrap()
//        .l()
//        .unwrap();
//    let view = env
//        .call_method(window, "getDecorView", "()Landroid/view/View;", &[])
//        .unwrap()
//        .l()
//        .unwrap();
//    let view_class = env.find_class("android/view/View").unwrap();
//    let flag_fullscreen = env
//        .get_static_field(view_class, "SYSTEM_UI_FLAG_FULLSCREEN", "I")
//        .unwrap()
//        .i()
//        .unwrap();
//    let flag_hide_navigation = env
//        .get_static_field(view_class, "SYSTEM_UI_FLAG_HIDE_NAVIGATION", "I")
//        .unwrap()
//        .i()
//        .unwrap();
//    let flag_immersive_sticky = env
//        .get_static_field(view_class, "SYSTEM_UI_FLAG_IMMERSIVE_STICKY", "I")
//        .unwrap()
//        .i()
//        .unwrap();
//    let flag = flag_fullscreen | flag_hide_navigation | flag_immersive_sticky;
//    match env.call_method(
//        view,
//        "setSystemUiVisibility",
//        "(I)V",
//        &[jni::objects::JValue::Int(flag)],
//    ) {
//        Err(_) => log::warn!("Failed to enable immersive mode"),
//        Ok(_) => {}
//    }
//    env.exception_clear().unwrap();
//}