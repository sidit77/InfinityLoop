use std::fmt::Debug;
use enum_iterator::IntoEnumIterator;
use png::{BitDepth, ColorType, Transformations};
use crate::{Context, DataType, InternalFormat, MipmapLevels, Texture, TextureType};
use crate::opengl::{Format, Region3d};
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

pub fn generate_tile_texture(ctx: &Context) -> Result<Texture, String> {
    let textures = [
        &include_bytes!(concat!(env!("OUT_DIR"), "/hex.png"))[..],
        &include_bytes!(concat!(env!("OUT_DIR"), "/tile0.png"))[..],
        &include_bytes!(concat!(env!("OUT_DIR"), "/tile01.png"))[..],
        &include_bytes!(concat!(env!("OUT_DIR"), "/tile02.png"))[..],
        &include_bytes!(concat!(env!("OUT_DIR"), "/tile03.png"))[..],
        &include_bytes!(concat!(env!("OUT_DIR"), "/tile012.png"))[..],
        &include_bytes!(concat!(env!("OUT_DIR"), "/tile024.png"))[..],
        &include_bytes!(concat!(env!("OUT_DIR"), "/tile0134.png"))[..]
    ];

    let texture = Texture::new(ctx, TextureType::Texture2dArray(64, 64, textures.len() as u32),
                               InternalFormat::R8, MipmapLevels::Full)?;

    let mut buf = vec![0; 64 * 64 * 1];

    for (i, png) in textures.iter().enumerate() {
        let mut decoder = png::Decoder::new(*png);
        decoder.set_transformations(Transformations::EXPAND);
        let mut reader = decoder.read_info().map_err(|e|e.to_string())?;
        let info = reader.next_frame(&mut buf).map_err(|e|e.to_string())?;

        assert_eq!(info.bit_depth, BitDepth::Eight);
        assert_eq!(info.color_type, ColorType::Grayscale);
        assert_eq!(info.width, 64);
        assert_eq!(info.height, 64);

        texture.fill_region_3d(0, Region3d::slice2d(info.width, info.height, i as u32), Format::R, DataType::U8, &buf[..info.buffer_size()]);
    }

    texture.generate_mipmaps();
    Ok(texture)
}