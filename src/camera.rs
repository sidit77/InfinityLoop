use std::ops::Deref;
use std::time::{Duration};
use glam::*;
use instant::Instant;
use crate::types::Angle;

#[derive(Debug, Copy, Clone, PartialEq)]
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
    velocity: Vec2,
    last_position_update: Instant,
    captured: bool
}

const TIME_STEP: Duration = Duration::from_millis(20);

fn update_vecity(velocity: Vec2) -> Vec2 {
    let percent_loss = velocity.length() * 0.1;
    let linear_loss = f32::min(velocity.length(), 0.1);
    velocity - velocity.normalize_or_zero() * f32::max(percent_loss, linear_loss)
}

impl AnimatedCamera {

    pub fn update_required(&self) -> bool {
        !self.captured && self.velocity.length_squared() > 0.0
    }

    pub fn update(&mut self, delta: Duration) {
        if self.update_required() {

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

    }

    pub fn move_by(&mut self, offset: Vec2) {
        self.parent.position += offset;

        let time = Instant::now();
        self.velocity = Vec2::lerp(self.velocity, offset / (time - self.last_position_update).as_secs_f32(), 0.5);
        self.last_position_update = time;
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
            captured: false
        }
    }
}

impl From<AnimatedCamera> for Camera {
    fn from(amin: AnimatedCamera) -> Self {
        amin.parent
    }
}

