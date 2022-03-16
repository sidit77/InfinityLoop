use std::io::Read;
use glow::{HasContext, PixelUnpackData};
use png::{BitDepth, ColorType, Transformations};
use crate::Context;
use crate::opengl::TextureTarget;

type GlowTexture = glow::Texture;

pub struct Texture {
    ctx: Context,
    id: GlowTexture,
    target: TextureTarget
}

impl Texture {

    pub fn new(ctx: &Context, target: TextureTarget) -> Result<Self, String> {
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

    pub fn load_png<R: Read>(ctx: &Context, png: R) -> Result<Self, String> {
        let tex = Self::new(ctx, TextureTarget::Texture2D)?;
        ctx.bind_texture(0, &tex);
        let gl = ctx.raw();
        unsafe {
            let mut decoder = png::Decoder::new(png);
            decoder.set_transformations(Transformations::EXPAND);
            let mut reader = decoder.read_info().map_err(|e|e.to_string())?;
            let mut buf = vec![0; reader.output_buffer_size()];
            let info = reader.next_frame(&mut buf).map_err(|e|e.to_string())?;
            assert_eq!(info.bit_depth, BitDepth::Eight);
            assert_eq!(info.color_type, ColorType::Rgb);
            let width = info.width as i32;
            let height = info.height as i32;
            let levels = 1 + f32::floor(f32::log2(f32::max(width as f32, height as f32))) as i32;
            gl.tex_storage_2d(tex.target.raw(), levels, glow::RGB8, width, height);
            gl.tex_sub_image_2d(tex.target.raw(), 0, 0, 0, width, height, glow::RGB, glow::UNSIGNED_BYTE, PixelUnpackData::Slice(&buf[..info.buffer_size()]));
            gl.generate_mipmap(tex.target.raw());
            gl.tex_parameter_i32(tex.target.raw(), glow::TEXTURE_WRAP_S, glow::CLAMP_TO_EDGE as i32);
            gl.tex_parameter_i32(tex.target.raw(), glow::TEXTURE_WRAP_T, glow::CLAMP_TO_EDGE as i32);
            Ok(tex)
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