use std::io::Read;
use std::num::NonZeroU32;
use bytemuck::Pod;
use glow::{HasContext, PixelUnpackData};
use png::{BitDepth, ColorType, Transformations};
use crate::{Context, DataType};
use crate::opengl::{Format, InternalFormat, TextureTarget};

type GlowTexture = glow::Texture;

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


#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum TextureType {
    Texture2d(u32, u32),
    Texture2dArray(u32, u32, u32)
}

impl TextureType {

    pub fn target(self) -> TextureTarget {
        match self {
            TextureType::Texture2d(_, _) => TextureTarget::Texture2D,
            TextureType::Texture2dArray(_, _, _) => TextureTarget::Texture2DArray
        }
    }

    fn max(self) -> u32 {
        match self {
            TextureType::Texture2d(w, h) => w.max(h),
            TextureType::Texture2dArray(w, h, _) => w.max(h),
        }
    }

    pub fn get_levels(self, levels: MipmapLevels) -> u32 {
        match levels {
            MipmapLevels::Full => 1 + f32::floor(f32::log2(self.max() as f32)) as u32,
            MipmapLevels::None => 1,
            MipmapLevels::Custom(val) => val.get()
        }
    }

}

pub struct Texture {
    ctx: Context,
    id: GlowTexture,
    target: TextureTarget
}

impl Texture {

    fn empty(ctx: &Context, target: TextureTarget) -> Result<Self, String> {
        let gl = ctx.raw();
        unsafe {
            let id = gl.create_texture()?;
            Ok(Self {
                ctx: ctx.clone(),
                id,
                target
            })
        }
    }

    pub fn new(ctx: &Context, tex_type: TextureType, format: InternalFormat, levels: MipmapLevels) -> Result<Self, String> {
        let tex = Self::empty(ctx, tex_type.target())?;
        ctx.bind_texture(0, &tex);
        let gl = ctx.raw();
        unsafe {
            let levels= tex_type.get_levels(levels) as i32;
            match tex_type {
                TextureType::Texture2d(w, h) =>
                    gl.tex_storage_2d(tex.target.raw(), levels, format.raw(), w as i32, h as i32),
                TextureType::Texture2dArray(w, h, d) =>
                    gl.tex_storage_3d(tex.target.raw(), levels, format.raw(), w as i32, h as i32, d as i32)
            }
        }
        Ok(tex)
    }

    pub fn load_png<R: Read>(ctx: &Context, png: R) -> Result<Self, String> {
        let mut decoder = png::Decoder::new(png);
        decoder.set_transformations(Transformations::EXPAND);
        let mut reader = decoder.read_info().map_err(|e|e.to_string())?;
        let mut buf = vec![0; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf).map_err(|e|e.to_string())?;

        assert_eq!(info.bit_depth, BitDepth::Eight);
        assert_eq!(info.color_type, ColorType::Grayscale);

        let tex = Self::new(ctx, TextureType::Texture2d(info.width, info.height),
                            InternalFormat::R8, MipmapLevels::Full)?;
        tex.fill_region_2d(0, Region2d::dimensions(info.width, info.height), Format::R, DataType::U8, &buf[..info.buffer_size()]);
        tex.generate_mipmaps();
        let gl = ctx.raw();
        unsafe {
            gl.tex_parameter_i32(tex.target.raw(), glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(tex.target.raw(), glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            Ok(tex)
        }
    }

    pub fn fill_region_3d<T: Pod>(&self, level: u32, region: Region3d, format: Format, data_type: DataType, data: &[T]) {
        let gl = self.ctx.raw();
        unsafe {
            gl.tex_sub_image_3d(self.target.raw(), level as i32, region.x as i32, region.y as i32, region.z as i32,
                                region.width as i32, region.height as i32, region.depth as i32, format.raw(),
                                data_type.raw(), PixelUnpackData::Slice(bytemuck::cast_slice(data)));

        }
    }

    pub fn fill_region_2d<T: Pod>(&self, level: u32, region: Region2d, format: Format, data_type: DataType, data: &[T]) {
        let gl = self.ctx.raw();
        unsafe {
            gl.tex_sub_image_2d(self.target.raw(), level as i32, region.x as i32, region.y as i32,
                                region.width as i32, region.height as i32, format.raw(),
                                data_type.raw(), PixelUnpackData::Slice(bytemuck::cast_slice(data)));

        }
    }

    pub fn generate_mipmaps(&self) {
        let gl = self.ctx.raw();
        unsafe {
            gl.generate_mipmap(self.target.raw());
        }
    }

    pub fn raw(&self) -> GlowTexture {
        self.id
    }

    pub fn target(&self) -> TextureTarget {
        self.target
    }

}

impl Drop for Texture {

    fn drop(&mut self) {
        let gl = self.ctx.raw();
        unsafe {
            gl.delete_texture(self.id);
        }
    }

}
