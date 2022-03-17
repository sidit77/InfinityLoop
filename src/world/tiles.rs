use std::fmt::Debug;
use enum_iterator::IntoEnumIterator;
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
    pub fn model(self) -> usize {
        match self {
            TileType::Tile0 => 1,
            TileType::Tile01 => 2,
            TileType::Tile02 => 3,
            TileType::Tile03 => 4,
            TileType::Tile012 => 5,
            TileType::Tile024 => 6,
            TileType::Tile0134 => 7,
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

    pub fn rotate_by(self, d: u8) -> Self {
        match self {
            TileConfig::Empty => TileConfig::Empty,
            TileConfig::Tile(t, r) => TileConfig::Tile(t, (r + d) % 6),
        }
    }

    pub fn with_rotation(self, r: u8) -> Self {
        match self {
            TileConfig::Empty => TileConfig::Empty,
            TileConfig::Tile(t, _) => TileConfig::Tile(t, r)
        }
    }

    pub fn model(self) -> usize {
        match self {
            TileConfig::Empty => panic!("TileConfig::Empty has no model"),
            TileConfig::Tile(t, _) => t.model(),
        }
    }
}

