use glow::HasContext;
use crate::opengl::{Context, DataType};

type GlowVertexArray = glow::VertexArray;

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum VertexArrayAttribute {
    Float(DataType, u32, bool),
    Integer(DataType, u32),
    Double(DataType, u32)
}

impl VertexArrayAttribute {

    fn size(self) -> u32 {
        match self {
            VertexArrayAttribute::Float(data_type, size, _) => data_type.size() * size,
            VertexArrayAttribute::Integer(data_type, size) => data_type.size() * size,
            VertexArrayAttribute::Double(data_type, size) => data_type.size() * size,
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

    pub fn set_bindings(&self, bindings: &[VertexArrayAttribute]) {
        let gl = self.ctx.raw();
        let stride = bindings.iter().map(|b|b.size()).sum::<u32>() as i32;
        let mut offset = 0;
        for (id, binding) in bindings.iter().copied().enumerate() {
            unsafe {
                gl.enable_vertex_attrib_array(id as u32);
                match binding {
                    VertexArrayAttribute::Float(data_type, size, normalized) => {
                        debug_assert!(matches!(data_type, DataType::I8 | DataType::U8 | DataType::I16 | DataType::U16 | DataType::I32 | DataType::U32 | DataType::F16 | DataType::F32 | DataType::F64));
                        gl.vertex_attrib_pointer_f32(id as u32, size as i32, data_type.raw(), normalized, stride, offset);
                    }
                    VertexArrayAttribute::Integer(data_type, size) => {
                        debug_assert!(matches!(data_type, DataType::I8 | DataType::U8 | DataType::I16 | DataType::U16 | DataType::I32 | DataType::U32));
                        gl.vertex_attrib_pointer_i32(id as u32, size as i32, data_type.raw(), stride, offset);
                    }
                    VertexArrayAttribute::Double(data_type, size) => {
                        debug_assert!(matches!(data_type, DataType::F64));
                        gl.vertex_attrib_pointer_f64(id as u32, size as i32, data_type.raw(), stride, offset);
                    }
                }
                offset += binding.size() as i32;
            }
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