use glam::*;

pub struct Camera {
    pub position: Vec2,
    pub aspect: f32,
    pub scale: f32,
    pub rotation: f32
}

impl Default for Camera {
    fn default() -> Self {
        Self{
            position: vec2(0.0,0.0),
            aspect: 1.0,
            scale: 350.0,
            rotation: 0.0
        }
    }
}

impl Camera {

    pub fn calc_aspect(&mut self, width: u32, height: u32){
        self.aspect = width as f32 / height as f32;
    }

    pub fn to_matrix(&self) -> Mat4 {


        Mat4::orthographic_rh(-(self.scale * self.aspect),
                              (self.scale * self.aspect),
                              -self.scale,
                              self.scale, 0.0, 100.0) *
        Mat4::from_rotation_z(-self.rotation) *
        Mat4::from_translation(-self.position.extend(0.0))

    }
}
