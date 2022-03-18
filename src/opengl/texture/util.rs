use std::num::NonZeroU32;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Region2d {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32
}

impl Region2d {
    pub fn dimensions(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            width,
            height
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Region3d {
    pub x: u32,
    pub y: u32,
    pub z: u32,
    pub width: u32,
    pub height: u32,
    pub depth: u32
}

impl Region3d {
    pub fn slice2d(width: u32, height: u32, index: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            z: index,
            width,
            height,
            depth: 1
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum MipmapLevels {
    Full,
    None,
    Custom(NonZeroU32)
}
