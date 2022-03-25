use glow::HasContext;
use crate::{Buffer, BufferTarget};
use crate::opengl::{Context, DataType};

type GlowVertexArray = glow::VertexArray;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VertexArrayAttribute {
    Float(u32, DataType, u32, bool),
    Integer(u32, DataType, u32),
    Double(u32, DataType, u32)
}

impl VertexArrayAttribute {

    fn size(self) -> u32 {
        match self {
            VertexArrayAttribute::Float(_, data_type, size, _) => data_type.size() * size,
            VertexArrayAttribute::Integer(_, data_type, size) => data_type.size() * size,
            VertexArrayAttribute::Double(_, data_type, size) => data_type.size() * size,
        }
    }

    fn id(self) -> u32 {
        match self {
            VertexArrayAttribute::Float(id, _, _, _) => id,
            VertexArrayAttribute::Integer(id, _, _) => id,
            VertexArrayAttribute::Double(id, _, _) => id,
        }
    }

}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VertexStepMode {
    Vertex,
    Instance
}

impl VertexStepMode {
    fn divisor(self) -> u32 {
        match self {
            VertexStepMode::Vertex => 0,
            VertexStepMode::Instance => 1
        }
    }
}

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

    pub fn set_bindings(&self, buffer: &Buffer, mode: VertexStepMode, bindings: &[VertexArrayAttribute]) {
        debug_assert_eq!(buffer.target(), BufferTarget::Array);
        self.ctx.bind_buffer(buffer);
        let gl = self.ctx.raw();
        let stride = bindings.iter().map(|b|b.size()).sum::<u32>() as i32;
        let mut offset = 0;
        for binding in bindings {
            unsafe {
                gl.enable_vertex_attrib_array(binding.id());
                match *binding {
                    VertexArrayAttribute::Float(id, data_type, size, normalized) => {
                        debug_assert!(matches!(data_type, DataType::I8 | DataType::U8 | DataType::I16 | DataType::U16 | DataType::I32 | DataType::U32 | DataType::F16 | DataType::F32 | DataType::F64));
                        gl.vertex_attrib_pointer_f32(id, size as i32, data_type.raw(), normalized, stride, offset);
                    }
                    VertexArrayAttribute::Integer(id, data_type, size) => {
                        debug_assert!(matches!(data_type, DataType::I8 | DataType::U8 | DataType::I16 | DataType::U16 | DataType::I32 | DataType::U32));
                        gl.vertex_attrib_pointer_i32(id, size as i32, data_type.raw(), stride, offset);
                    }
                    VertexArrayAttribute::Double(id, data_type, size) => {
                        debug_assert!(matches!(data_type, DataType::F64));
                        gl.vertex_attrib_pointer_f64(id, size as i32, data_type.raw(), stride, offset);
                    }
                }
                gl.vertex_attrib_divisor(binding.id(), mode.divisor());
                offset += binding.size() as i32;
            }
        }
    }

    pub fn raw(&self) -> GlowVertexArray {
        self.id
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