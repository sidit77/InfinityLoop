use glam::*;
use crate::Angle;

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

}
