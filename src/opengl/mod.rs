#![allow(dead_code)]

mod shader;
mod vertex_array;
mod enums;
mod buffer;
mod texture;
mod framebuffer;

use std::ops::Range;
pub use shader::*;
pub use vertex_array::*;
pub use buffer::*;
pub use enums::*;
pub use texture::*;
pub use framebuffer::*;

use std::rc::Rc;
use glow::{COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT, HasContext};
use crate::types::Rgba;

type GlowContext = glow::Context;

#[derive(Debug, Clone)]
pub struct Context(Rc<GlowContext>);

impl Context {

    #[cfg(not(target_arch = "wasm32"))]
    pub fn from_loader_function(func: impl FnMut(&str) -> *const core::ffi::c_void, ) -> Self {
        Self::from_glow(unsafe { GlowContext::from_loader_function(func) })
    }

    /// Create an instance from a WebGL2 context
    #[cfg(target_arch = "wasm32")]
    pub fn from_webgl2_context(gl: web_sys::WebGl2RenderingContext) -> Self {
        Self::from_glow(GlowContext::from_webgl2_context(gl))
    }

    fn from_glow(gl: GlowContext) -> Self {
        Context(Rc::new(gl))
    }

    pub fn raw(&self) -> &GlowContext {
        &self.0
    }

    pub fn clear(&self, color: impl Into<Rgba<f32>>) {
        let gl = self.raw();
        let color = color.into();
        unsafe {
            gl.clear_color(color.r, color.g, color.b, color.a);
            gl.clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
        }

    }

    pub fn viewport(&self, x: i32, y: i32, width: i32, height: i32) {
        let gl = self.raw();
        unsafe {
            gl.viewport(x, y, width, height)
        }
    }

    pub fn use_program<'a>(&self, program: impl Into<Option<&'a ShaderProgram>>) {
        let gl = self.raw();
        unsafe {
            gl.use_program(program.into().map(|p| p.raw()))
        }
    }

    pub fn use_vertex_array<'a>(&self, vertex_array: impl Into<Option<&'a VertexArray>>) {
        let gl = self.raw();
        unsafe {
            gl.bind_vertex_array(vertex_array.into().map(|p| p.raw()));
        }
    }

    pub fn use_framebuffer<'a>(&self, framebuffer: impl Into<Option<&'a Framebuffer>>) {
        let gl = self.raw();
        unsafe {
            gl.bind_framebuffer(glow::FRAMEBUFFER, framebuffer.into().map(|p| p.raw()));
        }
    }

    pub fn draw_arrays(&self, primitive_type: PrimitiveType, first: i32, count: i32) {
        let gl = self.raw();
        unsafe {
            gl.draw_arrays(primitive_type.raw(), first, count);
        }
    }

    pub fn draw_elements_range(&self, primitive_type: PrimitiveType, index_type: DataType, range: Range<i32>) {
        debug_assert!(matches!(index_type, DataType::U8 | DataType::U16 | DataType::U32));
        let gl = self.raw();
        let count = range.len() as i32;
        let first = range.start;
        unsafe {
            gl.draw_elements(primitive_type.raw(), count, index_type.raw(), first * index_type.size() as i32);
        }
    }

    pub fn bind_buffer(&self, buffer: &Buffer) {
        let gl = self.raw();
        unsafe {
            gl.bind_buffer(buffer.target().raw(), Some(buffer.raw()));
        }
    }

    pub fn bind_renderbuffer(&self, buffer: &Renderbuffer) {
        let gl = self.raw();
        unsafe {
            gl.bind_renderbuffer(glow::RENDERBUFFER, Some(buffer.raw()));
        }
    }

    pub fn bind_texture(&self, slot: u32, texture: &Texture) {
        let gl = self.raw();
        unsafe {
            gl.active_texture(glow::TEXTURE0 + slot);
            gl.bind_texture(texture.target().raw(), Some(texture.raw()));
        }
    }

    pub fn set_blend_state(&self, state: impl Into<Option<BlendState>>){
        let gl = self.raw();
        unsafe {
            match state.into() {
                None => gl.disable(glow::BLEND),
                Some(state) => {
                    gl.enable(glow::BLEND);
                    gl.blend_func(state.src.raw(), state.dst.raw());
                    gl.blend_equation(state.equ.raw())
                }
            }
        }
    }

}

