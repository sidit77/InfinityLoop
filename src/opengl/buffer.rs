use bytemuck::Pod;
use glow::HasContext;
use crate::opengl::{Context, GlResult};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum BufferTarget {
    Array = glow::ARRAY_BUFFER,
    AtomicCounter = glow::ATOMIC_COUNTER_BUFFER,
    CopyRead = glow::COPY_READ_BUFFER,
    CopyWrite = glow::COPY_WRITE_BUFFER,
    DispatchIndirect = glow::DISPATCH_INDIRECT_BUFFER,
    DrawIndirect = glow::DRAW_INDIRECT_BUFFER,
    ElementArray = glow::ELEMENT_ARRAY_BUFFER,
    PixelPack = glow::PIXEL_PACK_BUFFER,
    PixelUnpack = glow::PIXEL_UNPACK_BUFFER,
    Query = glow::QUERY_BUFFER,
    ShaderStorage = glow::SHADER_STORAGE_BUFFER,
    Texture = glow::TEXTURE_BUFFER,
    TransformFeedback = glow::TRANSFORM_FEEDBACK_BUFFER,
    Uniform = glow::UNIFORM_BUFFER
}

impl BufferTarget {
    pub fn raw(self) -> u32 {
        self as u32
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum BufferUsage {
    StreamDraw = glow::STREAM_DRAW,
    StreamRead = glow::STREAM_READ,
    StreamCopy = glow::STREAM_COPY,
    DynamicDraw = glow::DYNAMIC_DRAW,
    DynamicRead = glow::DYNAMIC_READ,
    DynamicCopy = glow::DYNAMIC_COPY,
    StaticDraw = glow::STATIC_DRAW,
    StaticRead = glow::STATIC_READ,
    StaticCopy = glow::STATIC_COPY
}

impl BufferUsage {
    pub fn raw(self) -> u32 {
        self as u32
    }
}

type GlowBuffer = glow::Buffer;

pub struct Buffer {
    ctx: Context,
    id: GlowBuffer,
    target: BufferTarget
}

impl Buffer {

    pub fn new(ctx: &Context, target: BufferTarget) -> GlResult<Self> {
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

    pub fn set_data<T: Pod>(&self, data: &[T], usage: BufferUsage){
        self.ctx.bind_buffer(self);
        let data = bytemuck::cast_slice(data);
        let gl = self.ctx.raw();
        unsafe {
            gl.buffer_data_u8_slice(self.target.raw(), data, usage.raw());
        }
    }

    pub fn set_sub_data<T: Pod>(&self, offset: usize, data: &[T]){
        self.ctx.bind_buffer(self);
        let data = bytemuck::cast_slice(data);
        let gl = self.ctx.raw();
        unsafe {
            gl.buffer_sub_data_u8_slice(self.target.raw(), (std::mem::size_of::<T>() * offset) as i32, data);
        }
    }


    pub fn raw(&self) -> GlowBuffer {
        self.id
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