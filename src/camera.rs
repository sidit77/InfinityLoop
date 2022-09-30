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
pub struct AnimatedCamera {
    pub parent: Camera,
    new_scale: f32,
    zoom_center: Vec2,

    velocity: Vec2,
    last_position_update: Instant,
    captured: bool,

}

const TIME_STEP: Duration = Duration::from_millis(20);

fn update_vecity(velocity: Vec2) -> Vec2 {
    let percent_loss = velocity.length() * 0.1;
    let linear_loss = f32::min(velocity.length(), 0.1);
    velocity - velocity.normalize_or_zero() * f32::max(percent_loss, linear_loss)
}

fn lerp(a: f32, b: f32, v: f32) -> f32 {
    a + (b - a) * v.clamp(0.0, 1.0)
}

impl AnimatedCamera {

    pub fn update_required(&self) -> bool {
         self.moving() || self.zooming()
    }

    fn moving(&self) -> bool {
        !self.captured && self.velocity.length_squared() > 0.0
    }

    fn zooming(&self) -> bool {
        self.parent.scale != self.new_scale
    }

    pub fn update(&mut self, delta: Duration) {
        if self.moving() {

            let now = Instant::now();
            while (now - self.last_position_update) >= TIME_STEP {
                self.last_position_update += TIME_STEP;
                self.velocity = update_vecity(self.velocity);
            }
            let next = (now - self.last_position_update).as_secs_f32() / TIME_STEP.as_secs_f32();
            self.parent.position = Vec2::lerp(
                self.parent.position + self.velocity * delta.as_secs_f32(),
                self.parent.position + update_vecity(self.velocity) * delta.as_secs_f32(),
                next);

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
        self.velocity = Vec2::lerp(self.velocity, offset / (time - self.last_position_update).as_secs_f32(), 0.5);
        self.last_position_update = time;
    }

    fn set_scale(&mut self, center: Vec2, level: f32) {
        let old = self.to_world_coords(center);
        self.parent.scale = level;
        let new = self.to_world_coords(center);
        self.parent.position += old - new;
    }

    pub fn zoom(&mut self, center: Vec2, amount: f32, animate: bool) {
        self.new_scale = self.new_scale.sub(amount * (self.new_scale / 10.0)).max(1.0);
        self.zoom_center = center;
        if !animate {
            self.set_scale(self.zoom_center, self.new_scale);
        }
    }

    pub fn capture(&mut self) {
        self.last_position_update = Instant::now();
        self.velocity = Vec2::ZERO;
        self.captured = true;
    }

    pub fn release(&mut self) {
        self.captured = false;
        //if self.last_position_update.elapsed().as_secs_f32() > 0.25 {
        //    self.velocity = Vec2::ZERO;
        //}
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
            velocity: Vec2::ZERO,
            last_position_update: Instant::now(),
            captured: false,
            new_scale: camera.scale,
            zoom_center: Vec2::ZERO
        }
    }
}

impl From<AnimatedCamera> for Camera {
    fn from(amin: AnimatedCamera) -> Self {
        amin.parent
    }
}

