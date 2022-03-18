use std::fmt::Debug;
use enum_iterator::IntoEnumIterator;
use glam::Vec2;
use sdf2d::{Constant, Ops, Sdf, Shapes};
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
    let mut builder = ArrayTextureBuilder::new(ctx, 64, 64, 8, -5.0)?;

    let a = 0.75;
    let g = 0.75 * f32::tan(f32::to_radians(30.0));

    let hexagon = Shapes::hexagon(a).rotate(f32::to_radians(90.0));
    builder.fill_layer(0, hexagon);

    let tile0 = Shapes::circle(0.45)
        .subtract(Shapes::circle(0.25))
        .union(Shapes::rectangle(0.1, 0.25)
            .translate(0.0, -0.5)
            .rotate(f32::to_radians(30.0)));
    builder.fill_layer(TileType::Tile0.model() as u32, tile0);

    let tile01 = Shapes::circle(g + 0.1)
        .subtract(Shapes::circle(g - 0.1))
        .translate(a, -g)
        .intersection(hexagon);
    builder.fill_layer(TileType::Tile01.model() as u32, tile01);

    let tile02 = Shapes::circle(3.0 * g + 0.1)
        .subtract(Shapes::circle(3.0 * g - 0.1))
        .translate(2.0 * a, 0.0)
        .intersection(hexagon);
    builder.fill_layer(TileType::Tile02.model() as u32, tile02);

    let tile03 = Shapes::rectangle(0.1, 0.75)
        .rotate(f32::to_radians(210.0));
    builder.fill_layer(TileType::Tile03.model() as u32, tile03);

    let tile012 = Constant::Empty
        .union(Shapes::circle(g + 0.1)
            .subtract(Shapes::circle(g - 0.1))
            .translate(a, -g))
        .union(Shapes::circle(g + 0.1)
            .subtract(Shapes::circle(g - 0.1))
            .translate(a,  g))
        .intersection(hexagon);
    builder.fill_layer(TileType::Tile012.model() as u32, tile012);

    let tile024 = Constant::Empty
        .union(Shapes::circle(3.0 * g + 0.1)
            .subtract(Shapes::circle(3.0 * g - 0.1))
            .translate(2.0 * a, 0.0))
        .union(Shapes::circle(3.0 * g + 0.1)
            .subtract(Shapes::circle(3.0 * g - 0.1))
            .translate(-a, 3.0 * g))
        .union(Shapes::circle(3.0 * g + 0.1)
            .subtract(Shapes::circle(3.0 * g - 0.1))
            .translate(-a, -3.0 * g))
        .intersection(hexagon);
    builder.fill_layer(TileType::Tile024.model() as u32, tile024);

    let tile0134 = Constant::Empty
        .union(Shapes::circle(g + 0.1)
            .subtract(Shapes::circle(g - 0.1))
            .translate(a, -g))
        .union(Shapes::circle(3.0 * g + 0.1)
            .subtract(Shapes::circle(3.0 * g - 0.1))
            .translate(a, 3.0 * g))
        .union(Shapes::circle(g + 0.1)
            .subtract(Shapes::circle(g - 0.1))
            .translate(-a, g))
        .union(Shapes::circle(3.0 * g + 0.1)
            .subtract(Shapes::circle(3.0 * g - 0.1))
            .translate(-a, -3.0 * g))
        .intersection(hexagon);
    builder.fill_layer(TileType::Tile0134.model() as u32, tile0134);

    Ok(builder.finalize())
}

struct ArrayTextureBuilder {
    texture: Texture,
    buffer: Vec<u8>,
    width: u32,
    height: u32,
    layers: u32,
    factor: f32
}

impl ArrayTextureBuilder {

    fn new(ctx: &Context, width: u32, height: u32, layers: u32, factor: f32) -> Result<Self, String> {
        let texture = Texture::new(ctx, TextureType::Texture2dArray(width, height, layers),
                                   InternalFormat::R8, MipmapLevels::Full)?;
        Ok(Self {
            texture,
            buffer: Vec::with_capacity((width * height) as usize),
            width,
            height,
            layers,
            factor
        })
    }

    fn fill_layer(&mut self, layer: u32, sdf: impl Sdf) {
        assert!(layer < self.layers);

        self.buffer.clear();
        let f = Vec2::new(self.width as f32, self.height as f32) * 0.5;
        for y in 0..self.height {
            for x in 0..self.width {
                let p = (Vec2::new(x as f32, y as f32) + Vec2::new(0.5, 0.5) - f) / f;
                let p = p * Vec2::new(1.0, -1.0);
                let d = self.factor * sdf.density(p);
                let h = u8::MAX as f32 * 0.5;
                self.buffer.push((h + d * h).clamp(u8::MIN as f32, u8::MAX as f32) as u8);
            }
        }

        self.texture.fill_region_3d(0, Region3d::slice2d(self.width, self.height, layer),
                                    Format::R, DataType::U8, self.buffer.as_slice());
    }

    fn finalize(self) -> Texture {
        self.texture.generate_mipmaps();
        self.texture
    }

}

