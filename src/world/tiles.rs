use std::fmt::Debug;
use std::ops::Range;
use enum_iterator::IntoEnumIterator;
use fastrand::Rng;
use lazy_static::lazy_static;
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
            TileType::Tile0 => [false, true, false, false, false, false],
            TileType::Tile01 => [true, true, false, false, false, false],
            TileType::Tile02 => [true, false, false, false, false, true],
            TileType::Tile03 => [false, true, false, false, true, false],
            TileType::Tile012 => [true, true, false, false, false, true],
            TileType::Tile024 => [false, true, false, true, false, true],
            TileType::Tile0134 => [true, true, false, true, true, false],
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
    pub fn from(index: usize) -> Self {
        match ELEMENT_TABLE.get(index) {
            None => unreachable!(),
            Some(elem) => *elem,
        }
    }

    pub fn random(rng: &Rng) -> Self {
        Self::from(rng.usize(0..ELEMENT_TABLE.len()))
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

    pub fn rotate_by(self, d: u8) -> Self {
        match self {
            TileConfig::Empty => TileConfig::Empty,
            TileConfig::Tile(t, r) => TileConfig::Tile(t, r + d).normalized(),
        }
    }

    pub fn index(&self) -> usize {
        match ELEMENT_TABLE.iter().position(|e| *e == self.normalized()) {
            None => unreachable!(),
            Some(i) => i,
        }
    }

    pub fn model(self) -> Range<i32> {
        match self {
            TileConfig::Empty => panic!("TileConfig::Empty has no model"),
            TileConfig::Tile(t, _) => t.model(),
        }
    }
}

type IndexSet = smallbitset::Set64;

lazy_static! {
    static ref ELEMENT_TABLE: Vec<TileConfig> = {
        let mut result = Vec::new();
        for tile_type in TileType::into_enum_iter() {
            for rotation in 0..6 {
                result.push(TileConfig::Tile(tile_type, rotation));
            }
        }
        result.push(TileConfig::Empty);
        result
    };
    static ref EMPTY_ELEMENT_INDEX: usize = {
        ELEMENT_TABLE
            .iter()
            .position(|x| *x == TileConfig::Empty)
            .expect("Cannot find the empty element in table")
    };
    static ref ADJACENCY_LISTS: Vec<[IndexSet; 6]> = {
        assert!(ELEMENT_TABLE.len() <= IndexSet::full().len());
        let mut result = Vec::new();
        for i in 0..ELEMENT_TABLE.len() {
            let mut lists = [IndexSet::empty(); 6];
            for j in 0..lists.len() {
                for k in 0..ELEMENT_TABLE.len() {
                    let elem1 = ELEMENT_TABLE[i].endings();
                    let elem2 = ELEMENT_TABLE[k].endings();

                    if elem1[j] == elem2[(j + 3) % elem2.len()] {
                        lists[j] = lists[j].insert(k as u8);
                    }
                }
            }
            result.push(lists);
        }
        result
    };
}
