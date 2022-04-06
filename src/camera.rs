use glam::*;
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
