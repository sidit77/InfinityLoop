
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



#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum BlendFactor {
    Zero = glow::ZERO,
    One = glow::ONE,
    SrcColor = glow::SRC_COLOR,
    OneMinusSrcColor = glow::ONE_MINUS_SRC_COLOR,
    DstColor = glow::DST_COLOR,
    OneMinusDstColor = glow::ONE_MINUS_DST_COLOR,
    SrcAlpha = glow::SRC_ALPHA,
    OneMinusSrcAlpha = glow::ONE_MINUS_SRC_ALPHA,
    DstAlpha = glow::DST_ALPHA,
    OneMinusDstAlpha = glow::ONE_MINUS_DST_ALPHA,
    ConstantColor = glow::CONSTANT_COLOR,
    OneMinusConstantColor = glow::ONE_MINUS_CONSTANT_COLOR,
    ConstantAlpha = glow::CONSTANT_ALPHA,
    OneMinusConstantAlpha = glow::ONE_MINUS_CONSTANT_ALPHA,
    SrcAlphaSaturate = glow::SRC_ALPHA_SATURATE,
    Src1Color = glow::SRC1_COLOR,
    OneMinusSrc1Color = glow::ONE_MINUS_SRC1_COLOR,
    Src1Alpha = glow::SRC1_ALPHA,
    OneMinusSrc1Alpha = glow::ONE_MINUS_SRC1_ALPHA
}

impl BlendFactor {
    pub fn raw(self) -> u32 {
        self as u32
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum BlendEquation {
    Add = glow::FUNC_ADD,
    Subtract = glow::FUNC_SUBTRACT,
    ReverseSubtract = glow::FUNC_REVERSE_SUBTRACT,
    Min = glow::MIN,
    Max = glow::MAX
}

impl BlendEquation {
    pub fn raw(self) -> u32 {
        self as u32
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct BlendState {
    pub src: BlendFactor,
    pub dst: BlendFactor,
    pub equ: BlendEquation
}
