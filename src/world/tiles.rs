use std::fmt::Debug;
use std::iter::once;
use std::ops::Range;
use enum_iterator::IntoEnumIterator;
use fastrand::Rng;
use crate::meshes::{MODEL1, MODEL2, MODEL3, MODEL4, MODEL5, MODEL6, MODEL7};
use crate::types::Angle;

#[derive(Debug, IntoEnumIterator, Copy, Clone, Eq, PartialEq)]
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
    pub fn model(self) -> Range<i32> {
        match self {
            TileType::Tile0 => MODEL1,
            TileType::Tile01 => MODEL2,
            TileType::Tile02 => MODEL6,
            TileType::Tile03 => MODEL3,
            TileType::Tile012 => MODEL4,
            TileType::Tile024 => MODEL7,
            TileType::Tile0134 => MODEL5,
        }
    }
    pub fn endings(self) -> [bool; 6] {
        match self {
            TileType::Tile0 => [false, false, false, false, false, true],
            TileType::Tile01 => [true, false, false, false, false, true],
            TileType::Tile02 => [false, true, false, false, false, true],
            TileType::Tile03 => [false, false, true, false, false, true],
            TileType::Tile012 => [true, true, false, false, false, true],
            TileType::Tile024 => [false, true, false, true, false, true],
            TileType::Tile0134 => [true, false, true, true, false, true],
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TileConfig {
    Empty,
    Tile(TileType, u8),
}

impl Default for TileConfig {
    fn default() -> Self {
        TileConfig::Empty
    }
}

impl TileConfig {

    pub fn random(rng: &Rng) -> Self {
        let mut iter = once(TileConfig::Empty)
            .chain(TileType::into_enum_iter()
                .flat_map(|t| (0..6)
                    .into_iter()
                    .map(move |r| TileConfig::Tile(t, r)))).collect::<Vec<_>>();
        iter[rng.usize(0..iter.len())]
    }

    pub fn endings(self) -> [bool; 6] {
        match self {
            TileConfig::Empty => [false; 6],
            TileConfig::Tile(tile_type, rotation) => {
                let mut endings = tile_type.endings();
                endings.rotate_right(rotation as usize);
                endings
            }
        }
    }

    pub fn rotation(self) -> u8 {
        match self {
            TileConfig::Empty => 0,
            TileConfig::Tile(_, r) => r,
        }
    }

    pub fn is_empty(self) -> bool {
        match self {
            TileConfig::Empty => true,
            TileConfig::Tile(_, _) => false
        }
    }

    pub fn angle(self) -> Angle {
        Angle::radians(-std::f32::consts::FRAC_PI_3 * self.rotation() as f32)
    }

    pub fn normalized(self) -> Self {
        match self {
            TileConfig::Empty => TileConfig::Empty,
            TileConfig::Tile(t, r) => TileConfig::Tile(t, r % 6),
        }
    }

    pub fn with_rotation(self, r: u8) -> Self {
        match self {
            TileConfig::Empty => TileConfig::Empty,
            TileConfig::Tile(t, _) => TileConfig::Tile(t, r)
        }
    }

    pub fn model(self) -> Range<i32> {
        match self {
            TileConfig::Empty => panic!("TileConfig::Empty has no model"),
            TileConfig::Tile(t, _) => t.model(),
        }
    }
}

