
#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum ShaderType {
    Vertex = glow::VERTEX_SHADER,
    Fragment = glow::FRAGMENT_SHADER,
    Geometry = glow::GEOMETRY_SHADER,
    TesselationControl = glow::TESS_CONTROL_SHADER,
    TesselationEvaluation = glow::TESS_EVALUATION_SHADER,
    Compute = glow::COMPUTE_SHADER
}

impl ShaderType {
    pub fn raw(self) -> u32 {
        self as u32
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum PrimitiveType {
    Points = glow::POINTS,
    LineStrip = glow::LINE_STRIP,
    LineLoop = glow::LINE_LOOP,
    Lines = glow::LINES,
    LineStripAdjacency = glow::LINE_STRIP_ADJACENCY,
    LinesAdjacency = glow::LINES_ADJACENCY,
    TriangleStrip = glow::TRIANGLE_STRIP,
    TriangleFan = glow::TRIANGLE_FAN,
    Triangles = glow::TRIANGLES,
    TriangleStripAdjacency = glow::TRIANGLE_STRIP_ADJACENCY,
    TrianglesAdjacency = glow::TRIANGLES_ADJACENCY,
    Patches = glow::PATCHES
}

impl PrimitiveType {
    pub fn raw(self) -> u32 {
        self as u32
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum DataType {
    I8 = glow::BYTE,
    U8 = glow::UNSIGNED_BYTE,
    I16 = glow::SHORT,
    U16 = glow::UNSIGNED_SHORT,
    I32 = glow::INT,
    U32 = glow::UNSIGNED_INT,
    F16 = glow::HALF_FLOAT,
    F32 = glow::FLOAT,
    F64 = glow::DOUBLE
}

impl DataType {
    pub fn raw(self) -> u32 {
        self as u32
    }

    pub fn size(self) -> u32 {
        match self {
            DataType::I8 => 1,
            DataType::U8 => 1,
            DataType::I16 => 2,
            DataType::U16 => 2,
            DataType::I32 => 4,
            DataType::U32 => 4,
            DataType::F16 => 1,
            DataType::F32 => 4,
            DataType::F64 => 8,
        }
    }

}