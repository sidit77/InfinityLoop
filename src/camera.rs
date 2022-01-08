use glam::*;
use crate::{Angle, math};

#[derive(Debug, Copy, Clone)]
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
            scale: 350.0,
            rotation: Angle::empty()
        }
    }
}

impl Camera {

    pub fn to_world_coords(self, screen_coords: Vec2) -> Vec2 {
        self.to_matrix()
            .inverse()
            .transform_point3(
                (2.0 * screen_coords - Vec2::ONE).extend(0.0))
            .xy()
    }

    pub fn to_matrix(self) -> Mat4 {
        Mat4::orthographic_rh(-(self.scale * self.aspect),
                              self.scale * self.aspect,
                              -self.scale,
                              self.scale, 0.0, 100.0) *
        Mat4::from_rotation_z(-self.rotation.to_radians()) *
        Mat4::from_translation(-self.position.extend(0.0))

    }

    pub fn lerp(mut self, other: Self, v: f32) -> Self {
        self.position = Vec2::lerp(self.position, other.position, v);
        self.scale = math::lerp(self.scale, other.scale, v);
        self.aspect = math::lerp(self.aspect, other.aspect, v);
        self.rotation = Angle::lerp(self.rotation, other.rotation, v);
        self
    }
}
