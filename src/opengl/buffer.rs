use bytemuck::Pod;
use glow::HasContext;
use crate::opengl::{BufferTarget, Context};

type GlowBuffer = glow::Buffer;

pub struct Buffer {
    ctx: Context,
    id: GlowBuffer,
    target: BufferTarget
}

impl Buffer {

    pub fn new(ctx: &Context, target: BufferTarget) -> Result<Self, String> {
        let gl = ctx.raw();
        unsafe {
            let id = gl.create_buffer()?;
            Ok(Self {
                ctx: ctx.clone(),
                id,
                target
            })
        }
    }

    pub fn set_data<T: Pod>(&self, data: &[T]){
        self.ctx.bind_buffer(self);
        let data = bytemuck::cast_slice(data);
        let gl = self.ctx.raw();
        unsafe {
            gl.buffer_data_u8_slice(self.target.raw(), data, glow::STATIC_DRAW);
        }
    }

    pub fn raw(&self) -> &GlowBuffer {
        &self.id
    }

    pub fn target(&self) -> BufferTarget {
        self.target
    }

}

impl Drop for Buffer {

    fn drop(&mut self) {
        let gl = self.ctx.raw();
        unsafe {
            gl.delete_buffer(self.id);
        }
    }

}