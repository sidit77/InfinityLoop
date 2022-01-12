use glow::HasContext;
use crate::opengl::Context;

type GlowVertexArray = glow::VertexArray;

pub struct VertexArray {
    ctx: Context,
    id: GlowVertexArray
}

impl VertexArray {

    pub fn new(ctx: &Context) -> Result<Self, String> {
        let gl = ctx.raw();
        unsafe {
            let id = gl.create_vertex_array()?;
            Ok(Self {
                ctx: ctx.clone(),
                id
            })
        }
    }

    pub fn raw(&self) -> &GlowVertexArray {
        &self.id
    }

}

impl Drop for VertexArray {

    fn drop(&mut self) {
        let gl = self.ctx.raw();
        unsafe {
            gl.delete_vertex_array(self.id);
        }
    }

}