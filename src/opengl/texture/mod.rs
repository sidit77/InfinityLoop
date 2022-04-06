mod util;
mod enums;

pub use util::*;
pub use enums::*;

use bytemuck::Pod;
use glow::{HasContext, PixelUnpackData};
use crate::opengl::{Context, DataType, GlResult};
use crate::types::Rgba;

type GlowTexture = glow::Texture;

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

    pub fn new(ctx: &Context, tex_type: TextureType, format: InternalFormat, levels: MipmapLevels) -> GlResult<Self> {
        let gl = ctx.raw();
        let tex = unsafe {
            let id = gl.create_texture()?;
            Self {
                ctx: ctx.clone(),
                id,
                target: tex_type.target()
            }
        };
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

    pub fn set_filter_mode(&self, mode: FilterMode) {
        let gl = self.ctx.raw();
        unsafe {
            gl.tex_parameter_i32(self.target.raw(), glow::TEXTURE_MIN_FILTER, mode.min.raw() as i32);
            gl.tex_parameter_i32(self.target.raw(), glow::TEXTURE_MAG_FILTER, mode.mag.raw() as i32);
        }
    }

    pub fn set_wrap_mode(&self, mode: WrapMode) {
        let gl = self.ctx.raw();
        unsafe {
            gl.tex_parameter_i32(self.target.raw(), glow::TEXTURE_WRAP_S, mode.s.raw() as i32);
            gl.tex_parameter_i32(self.target.raw(), glow::TEXTURE_WRAP_T, mode.t.raw() as i32);
            gl.tex_parameter_i32(self.target.raw(), glow::TEXTURE_WRAP_R, mode.r.raw() as i32);
        }
    }

    pub fn set_lod_bias(&self, bias: f32) {
        let gl = self.ctx.raw();
        unsafe {
            gl.tex_parameter_f32(self.target.raw(), glow::TEXTURE_LOD_BIAS, bias);
        }
    }

    pub fn set_border_color(&self, color: impl Into<Rgba<f32>>) {
        let gl = self.ctx.raw();
        unsafe {
            gl.tex_parameter_f32_slice(self.target.raw(), glow::TEXTURE_BORDER_COLOR, color.into().as_ref());
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
