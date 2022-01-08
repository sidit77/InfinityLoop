use std::time::Duration;
use miniquad::{Context, EventHandler, MouseButton};
use miniquad::date::now;

pub enum Event {
    WindowResize(f32, f32),
    Click(f32, f32)
}

pub trait EventHandlerMod {
    fn draw(&mut self, ctx: &mut Context, delta: Duration);
    fn event(&mut self, ctx: &mut Context, event: Event);
}

pub struct EventHandlerProxy<T: EventHandlerMod> {
    last_render: f64,
    handler: T
}

impl<T: EventHandlerMod> From<T> for EventHandlerProxy<T> {
    fn from(handler: T) -> Self {
        Self {
            last_render: now(),
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

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            let (w, h) = ctx.screen_size();
            self.handler.event(ctx, Event::Click(x / w, 1.0 - y / h))
        }
    }

}