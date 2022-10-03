use std::collections::VecDeque;
use std::ops::{Deref, Sub};
use std::time::{Duration};
use glam::*;
use instant::Instant;
use serde::{Serialize, Deserialize};
use crate::types::Angle;

#[derive(Debug, Copy, Clone, PartialEq, Serialize, Deserialize)]
pub struct Camera {
    pub position: Vec2,
    pub aspect: f32,
    pub scale: f32,
    pub rotation: Angle
}

impl Default for Camera {
    fn default() -> Self {
        Self{
            position: vec2(0.0,0.0),
            aspect: 1.0,
            scale: 1.0,
            rotation: Angle::empty()
        }
    }
}

impl Camera {

    pub fn to_world_coords(self, screen_coords: Vec2) -> Vec2 {
        self.to_matrix()
            .inverse()
            .transform_point2(2.0 * screen_coords - Vec2::ONE)
    }

    pub fn to_matrix(self) -> Mat3 {
        Mat3::from_scale(1.0 / (Vec2::new(self.aspect, 1.0) * self.scale)) *
        Mat3::from_angle(-self.rotation.to_radians()) *
        Mat3::from_translation(-self.position)
    }

}

#[derive(Debug, Copy, Clone, PartialEq)]
struct Vec2Animation {
    start_value: Vec2,
    start_time: Instant,
    duration: Duration,
    offset: Vec2
}

impl Vec2Animation {

    fn complete(&self) -> bool {
        self.start_time.elapsed() >= self.duration
    }

    fn current_value(&self) -> Vec2 {
        let elapsed = self.start_time.elapsed();
        let progress = f32::min(1.0, elapsed.as_secs_f32() / self.duration.as_secs_f32());
        self.start_value + self.offset * ease_out(progress)
    }

}

#[derive(Debug, Clone, PartialEq)]
pub struct AnimatedCamera {
    pub parent: Camera,
    new_scale: f32,
    zoom_center: Vec2,

    pos_amim: Option<Vec2Animation>,

    past_positions: VecDeque<(Vec2, Instant)>
}

fn ease_out(t: f32) -> f32 {
    let ease_power = 1.0 / f32::max(EASE, 0.2);
    1.0 - f32::powf(1.0 - t, ease_power)
}

fn lerp(a: f32, b: f32, v: f32) -> f32 {
    a + (b - a) * v.clamp(0.0, 1.0)
}

const EASE: f32 = 0.4;
const DECELERATION: f32 = 20.0;

impl AnimatedCamera {

    pub fn update_required(&self) -> bool {
        self.pos_amim.is_some() || self.zooming()
    }

    fn zooming(&self) -> bool {
        self.parent.scale != self.new_scale
    }

    pub fn update(&mut self, delta: Duration) {

        if let Some(anim) = self.pos_amim {
            self.parent.position = anim.current_value();
            if anim.complete() {
                self.pos_amim = None;
            }
        }

        if self.zooming() {
            let mut new_level = lerp(self.parent.scale, self.new_scale, 1.0 - f32::exp(-(8.0 * delta.as_secs_f32())));
            if (new_level - self.new_scale).abs() <= 0.01 {
                new_level = self.new_scale;
            }
            self.set_scale(self.zoom_center, new_level);
        }

    }

    pub fn move_by(&mut self, offset: Vec2) {
        self.parent.position += offset;

        let time = Instant::now();
        self.past_positions.push_back((self.parent.position, time));
        self.cull_positions(time);

    }

    fn set_scale(&mut self, center: Vec2, level: f32) {
        let old = self.to_world_coords(center);
        self.parent.scale = level;
        let new = self.to_world_coords(center);
        self.parent.position += old - new;
        if let Some(anim) = &mut self.pos_amim {
            anim.start_value += old - new;
        }
    }

    pub fn zoom(&mut self, center: Vec2, amount: f32, animate: bool) {
        self.new_scale = self.new_scale.sub(amount * (self.new_scale / 10.0)).max(1.0);
        self.zoom_center = center;
        if !animate {
            self.set_scale(self.zoom_center, self.new_scale);
        }
    }

    pub fn capture(&mut self) {
        self.past_positions.clear();
        self.pos_amim = None;
    }

    pub fn release(&mut self) {
        let time = Instant::now();
        self.cull_positions(time);

        if let (Some((p_start, t_start)), Some((p_end, t_end))) = (self.past_positions.front(), self.past_positions.back()) {
            let direction = *p_end - *p_start;
            let duration = *t_end - *t_start;

            if direction == Vec2::ZERO || duration.is_zero() {
                return;
            }

            let velocity = direction * (EASE / duration.as_secs_f32());
            let speed = velocity.length();

            let max_speed = f32::INFINITY;
            let limited_speed = f32::min(speed, max_speed);
            let limited_velocity = velocity * (limited_speed / speed);

            let deceleration_duration = limited_speed / (DECELERATION * EASE);
            let offset = limited_velocity * (deceleration_duration * 0.5);

            self.pos_amim = Some(Vec2Animation {
                start_value: self.parent.position,
                start_time: time,
                duration: Duration::from_secs_f32(deceleration_duration),
                offset
            });
        }

    }

    fn cull_positions(&mut self, now: Instant) {
        let outdated = self.past_positions
            .iter()
            .take_while(|(_, t)| now.duration_since(*t) > Duration::from_millis(50))
            .count();
        self.past_positions.drain(..outdated);
    }

}

impl Deref for AnimatedCamera {
    type Target = Camera;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

impl From<Camera> for AnimatedCamera {
    fn from(camera: Camera) -> Self {
        Self {
            parent: camera,
            new_scale: camera.scale,
            zoom_center: Vec2::ZERO,
            pos_amim: None,
            past_positions: VecDeque::new(),
        }
    }
}

impl From<AnimatedCamera> for Camera {
    fn from(amin: AnimatedCamera) -> Self {
        amin.parent
    }
}

