use glam::Vec2;

pub struct Triangle(pub Vec2, pub Vec2, pub Vec2);

impl Triangle {
    pub fn contains(&self, point: Vec2) -> bool{
        let d_x = point.x - self.2.x;
        let d_y = point.y - self.2.y;
        let d_x21 = self.2.x - self.1.x;
        let d_y12 = self.1.y - self.2.y;
        let d = d_y12 * (self.0.x - self.2.x) + d_x21 * (self.0.y - self.2.y);
        let s = d_y12 * d_x + d_x21 * d_y;
        let t = (self.2.y - self.0.y) * d_x + (self.0.x - self.2.x) * d_y;
        if d < 0.0 {
            return s <= 0.0 && t <= 0.0 && s + t >= d;
        }
        return s >= 0.0 && t >= 0.0 && s+t <= d;
    }
}

pub struct Hexagon {
    pub position: Vec2,
    pub rotation: f32,
    pub radius: f32
}

impl Default for Hexagon {
    fn default() -> Self {
        Self {
            position: Vec2::new(0.0,0.0),
            rotation: 0.0,
            radius: 1.0
        }
    }
}

impl Hexagon {

    pub fn contains(&self, point: Vec2) -> bool {
        if (point - self.position).length_squared() > self.radius {
            return false;
        }

        let from_id = |i: u32|{
            let (sin, cos) = f32::sin_cos(-self.rotation + std::f32::consts::FRAC_PI_3 * i as f32);
            Vec2::new(self.position.x + self.radius * sin, self.position.y + self.radius * cos)
        };

        for i in 0u32..4 {
            if Triangle(from_id(0), from_id(i + 1), from_id(i + 2)).contains(point){
                return true
            }
        }
        false
    }
}
