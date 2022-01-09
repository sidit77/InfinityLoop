use std::mem::take;
use std::time::Duration;
use glam::Vec2;
use miniquad::{Context, EventHandler, MouseButton};
use miniquad::date::now;

#[derive(Debug, Copy, Clone)]
pub enum Event {
    WindowResize(f32, f32),
    Click(Vec2),
    Drag(Vec2),
    DragEnd(Vec2),
    Touch,
    Zoom(Vec2, f32),
}

pub trait EventHandlerMod {
    fn draw(&mut self, ctx: &mut Context, delta: Duration);
    fn event(&mut self, ctx: &mut Context, event: Event);
}

#[derive(Copy, Clone)]
enum ClickState {
    Click(Vec2),
    Drag(Vec2, Vec2),
    None
}

impl Default for ClickState {
    fn default() -> Self {
        Self::None
    }
}

pub struct EventHandlerProxy<T: EventHandlerMod> {
    last_render: f64,
    last_mouse_pos: Vec2,
    click_state: ClickState,
    handler: T
}

impl<T: EventHandlerMod> From<T> for EventHandlerProxy<T> {
    fn from(handler: T) -> Self {
        Self {
            last_render: now(),
            last_mouse_pos: Default::default(),
            click_state: ClickState::None,
            handler
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
const SCROLL_SPEED: f32 = 4.0;

#[cfg(target_arch = "wasm32")]
const SCROLL_SPEED: f32 = 40.0;

impl<T: EventHandlerMod> EventHandler for EventHandlerProxy<T> {
    fn update(&mut self, _ctx: &mut Context) {
        if let ClickState::Drag(pos, _) = self.click_state {
            self.click_state = ClickState::Drag(pos, Vec2::ZERO);
        }
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
        let npos = normalize_mouse_pos(ctx, x, y);
        match self.click_state {
            ClickState::Click(pos) => if pixel_dist(ctx, npos, pos) > 10.0 {
                self.click_state = ClickState::Drag(npos, npos - pos);
                self.handler.event(ctx, Event::Drag(npos - pos));
            }
            ClickState::Drag(pos, _) => {
                self.click_state = ClickState::Drag(npos, npos - pos);
                self.handler.event(ctx, Event::Drag(npos - pos))
            },
            ClickState::None => {}
        }
        self.last_mouse_pos = npos;
    }

    fn mouse_wheel_event(&mut self, ctx: &mut Context, _x: f32, y: f32) {
        self.handler.event(ctx, Event::Zoom(self.last_mouse_pos, y / SCROLL_SPEED))
    }

    fn mouse_button_down_event(&mut self, ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        if button == MouseButton::Left {
            self.click_state = ClickState::Click(normalize_mouse_pos(ctx, x, y));
            self.handler.event(ctx, Event::Touch)
            //self.handler.event(ctx, Event::Click(normalize_mouse_pos(ctx, x, y)))
        }
    }

    fn mouse_button_up_event(&mut self, ctx: &mut Context, button: MouseButton, _x: f32, _y: f32) {
        if button == MouseButton::Left {
            match take(&mut self.click_state) {
                ClickState::Click(pos) => self.handler.event(ctx, Event::Click(pos)),
                ClickState::Drag(_, delta) => self.handler.event(ctx, Event::DragEnd(delta)),
                _ => {}
            }
        }
    }
}

fn pixel_dist(ctx: &Context, p1: Vec2, p2: Vec2) -> f32 {
    let s: Vec2 = ctx.screen_size().into();
    let p1 = p1 * s;
    let p2 = p2 * s;
    p1.distance(p2) / ctx.dpi_scale()
}

fn normalize_mouse_pos(ctx: &Context,  x: f32, y: f32) -> Vec2 {
    let (w, h) = ctx.screen_size();
    Vec2::new(x / w, 1.0 - y / h)
}