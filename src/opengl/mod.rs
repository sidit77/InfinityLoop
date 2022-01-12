use std::rc::Rc;
use glow::{COLOR_BUFFER_BIT, DEPTH_BUFFER_BIT, HasContext};
use crate::types::RGBA;

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

    pub fn clear(&self, color: impl Into<RGBA<f32>>) {
        let gl = self.raw();
        let color = color.into();
        unsafe {
            gl.clear_color(color.r, color.g, color.b, color.a);
            gl.clear(COLOR_BUFFER_BIT | DEPTH_BUFFER_BIT);
        }

    }

}