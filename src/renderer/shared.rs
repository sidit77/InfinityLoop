use glam::Vec2;
use sdf2d::{Constant, Ops, Sdf, Shapes};
use crate::Camera;
use crate::opengl::*;
use crate::world::TileType;

pub const TILE_RES: u32 = 128;
pub const TILE_RANGE: f32 = 6.0;

pub struct TileRenderResources {
    shader: ShaderProgram,
    camera_location: UniformLocation,
    textures: Texture,
}

impl TileRenderResources {

    pub fn new(ctx: &Context) -> anyhow::Result<Self> {
        let shader = ShaderProgram::new(ctx, &[
            &Shader::new(ctx, ShaderType::Vertex, include_str!("../shader/tiles.vert"))?,
            &Shader::new(ctx, ShaderType::Fragment, include_str!("../shader/tiles.frag"))?,
        ])?;
        ctx.use_program(&shader);
        let camera_location = shader.get_uniform("camera")?;
        let tex_location = shader.get_uniform("tex")?;
        let range_location = shader.get_uniform("range")?;
        ctx.set_uniform(&tex_location, 0);
        ctx.set_uniform(&range_location, 1.0 / TILE_RANGE);

        let textures = generate_tile_texture(ctx)?;

        Ok(Self {
            shader,
            camera_location,
            textures
        })
    }

    pub fn prepare(&self, ctx: &Context, camera: &Camera) {
        ctx.use_program(&self.shader);
        ctx.bind_texture(0, &self.textures);
        ctx.set_uniform(&self.camera_location, camera.to_matrix());
    }

}


fn generate_tile_texture(ctx: &Context) -> GlResult<Texture> {
    let mut builder = ArrayTextureBuilder::new(ctx, TILE_RES, TILE_RES, 8, -TILE_RANGE)?;

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

    fn new(ctx: &Context, width: u32, height: u32, layers: u32, factor: f32) -> GlResult<Self> {
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
        self.texture.set_filter_mode(FilterMode {
            min: MinFilter::LinearMipmapLinear,
            mag: MagFilter::Linear
        });
        self.texture.set_wrap_mode(WrapMode {
            s: TextureWrap::ClampToEdge,
            t: TextureWrap::ClampToEdge,
            r: TextureWrap::ClampToEdge
        });
        self.texture
    }

}