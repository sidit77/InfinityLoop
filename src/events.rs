use std::time::Duration;
use glam::Vec2;
use miniquad::{Context, EventHandler, MouseButton};
use miniquad::date::now;

pub enum Event {
    WindowResize(f32, f32),
    Click(Vec2),
    Zoom(Vec2, f32),
}

pub trait EventHandlerMod {
    fn draw(&mut self, ctx: &mut Context, delta: Duration);
    fn event(&mut self, ctx: &mut Context, event: Event);
}

pub struct EventHandlerProxy<T: EventHandlerMod> {
    last_render: f64,
    last_mouse_pos: Vec2,
    handler: T
}

impl<T: EventHandlerMod> From<T> for EventHandlerProxy<T> {
    fn from(handler: T) -> Self {
        Self {
            last_render: now(),
            last_mouse_pos: Default::default(),
            handler
        }
    }
}

impl<T: EventHandlerMod> EventHandler for EventHandlerProxy<T> {
    fn update(&mut self, _ctx: &mut Context) {

    }

    fn draw(&mut self, ctx: &mut Context) {
        let delta = Duration::from_secs_f64((now() - self.last_render).max(0.0));
        self.last_render = now();
        self.handler.draw(ctx, delta)
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        self.handler.event(ctx, Event::WindowResize(width, height))
    }

    fn mouse_motion_event(&mut self, ctx: &mut Context, x: f32, y: f32) {
        self.last_mouse_pos = normalize_mouse_pos(ctx, x, y);
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
        self.handler.event(ctx, Event::Zoom(self.last_mouse_pos, y / 4.0))
    }


    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            self.handler.event(ctx, Event::Click(normalize_mouse_pos(ctx, x, y)))
        }
    }

}

fn normalize_mouse_pos(ctx: &Context,  x: f32, y: f32) -> Vec2 {
    let (w, h) = ctx.screen_size();
    Vec2::new(x / w, 1.0 - y / h)
}