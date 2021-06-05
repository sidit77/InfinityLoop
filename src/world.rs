use std::ops::Range;
use glam::Vec2;
use crate::meshes::{MODEL1, MODEL2, MODEL6, MODEL3, MODEL4, MODEL7, MODEL5};

const SIN_FRAC_PI_6: f32 = 0.5;
const COS_FRAC_PI_6: f32 = 0.86602540378;

#[derive(Debug, Copy, Clone)]
pub enum TileType {
    Tile0,
    Tile01,
    Tile02,
    Tile03,
    Tile012,
    Tile024,
    Tile0134,
}

impl TileType {
    pub fn model(&self) -> Range<i32> {
        match self {
            TileType::Tile0 => MODEL1,
            TileType::Tile01 => MODEL2,
            TileType::Tile02 => MODEL6,
            TileType::Tile03 => MODEL3,
            TileType::Tile012 => MODEL4,
            TileType::Tile024 => MODEL7,
            TileType::Tile0134 => MODEL5
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct WorldElement {
    pub tile_type: TileType,
    pub rotation: u8
}

#[derive(Debug)]
pub struct World {
    rows: u32,
    width: u32,
    elements: Box<[Option<WorldElement>]>
}

impl World {

    fn new(rows: u32, width: u32) -> Self {
        Self {
            rows,
            width,
            elements: vec![None; (rows * width + (rows / 2)) as usize].into_boxed_slice()
        }
    }

    pub fn from_seed(seed: u64) -> Self{
        let rng = fastrand::Rng::with_seed(seed);
        let mut world = World::new(9, 5);

        for i in world.indices() {
            *world.get_element(i) = Some(WorldElement {
                tile_type: match rng.u8(0..7) {
                    0 => TileType::Tile0,
                    1 => TileType::Tile01,
                    2 => TileType::Tile02,
                    3 => TileType::Tile03,
                    4 => TileType::Tile012,
                    5 => TileType::Tile024,
                    6 => TileType::Tile0134,
                    _ => unreachable!()
                },
                rotation: 0
            });
        }

        world
    }


    pub fn indices(&self) -> Range<usize> {
        0..self.elements.len()
    }
    pub fn get_index(&self, x: u32, y: u32) -> Option<usize> {
        if y < self.rows && x < (self.width + (y % 2)) {
            Some((y * self.width + (y / 2) + x) as usize)
        } else {
            None
        }
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
    pub fn get_element(&mut self, index: usize) -> &mut Option<WorldElement> {
        &mut self.elements[index]
    }
}
