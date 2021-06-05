use std::ops::Range;
use glam::Vec2;

const SIN_FRAC_PI_6: f32 = 0.5;
const COS_FRAC_PI_6: f32 = 0.86602540378;

#[derive(Debug, Default, Copy, Clone)]
pub struct WorldElement {
    pub rotation: u8
}

#[derive(Debug)]
pub struct World {
    rows: u32,
    width: u32,
    elements: Box<[WorldElement]>
}

impl World {

    fn new(rows: u32, width: u32) -> Self {
        Self {
            rows,
            width,
            elements: vec![Default::default(); (rows * width + (rows / 2)) as usize].into_boxed_slice()
        }
    }

    pub fn from_seed(seed: u64) -> Self{
        let rng = fastrand::Rng::with_seed(seed);
        let world = World::new(9,5);



        world
    }


    pub fn indices(&self) -> Range<usize> {
        0..self.elements.len()
    }
    pub fn get_xy(&self, index: usize) -> (u32, u32) {
        let div = index as u32 / (2 * self.width + 1);
        let rem = index as u32 % (2 * self.width + 1);

        if rem < self.width {
            (rem, 2 * div)
        } else {
            (rem - self.width, 2 * div + 1)
        }
    }
    pub fn get_position(&self, index: usize) -> Vec2{
        let (x, y) = self.get_xy(index);
        let offset = -((y % 2) as f32) * COS_FRAC_PI_6;
        Vec2::new(
            (-0.5 * self.width as f32 - 1.0) + (2.0 * COS_FRAC_PI_6) * x as f32 + offset,
            (-0.5 * self.rows  as f32 - 1.0) + (1.0 + SIN_FRAC_PI_6) * y as f32
        )
    }
    pub fn get_size(&self) -> (f32, f32) {
        ((2.0 * COS_FRAC_PI_6) * self.width as f32, (1.0 + SIN_FRAC_PI_6) * self.rows as f32)
    }
    pub fn get_element(&mut self, index: usize) -> &mut WorldElement {
        &mut self.elements[index]
    }
}
