
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

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum TextureTarget {
    Texture1D = glow::TEXTURE_1D,
    Texture2D = glow::TEXTURE_2D,
    Texture3D = glow::TEXTURE_3D,
    Texture1DArray = glow::TEXTURE_1D_ARRAY,
    Texture2DArray = glow::TEXTURE_2D_ARRAY,
    TextureRectangle = glow::TEXTURE_RECTANGLE,
    TextureCubeMap = glow::TEXTURE_CUBE_MAP,
    TextureCubeMapArray = glow::TEXTURE_CUBE_MAP_ARRAY,
    TextureBuffer = glow::TEXTURE_BUFFER,
    Texture2DMultisample = glow::TEXTURE_2D_MULTISAMPLE,
    Texture2DMultisampleArray = glow::TEXTURE_2D_MULTISAMPLE_ARRAY,
}

impl TextureTarget {
    pub fn raw(self) -> u32 {
        self as u32
    }
}

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum InternalFormat {
    R8 = glow::R8,
    R8Snorm = glow::R8_SNORM,
    R16 = glow::R16,
    R16Snorm = glow::R16_SNORM,
    Rg8 = glow::RG8,
    Rg8Snorm = glow::RG8_SNORM,
    Rg16 = glow::RG16,
    Rg16Snorm = glow::RG16_SNORM,
    R3g3b2 = glow::R3_G3_B2,
    Rgb4 = glow::RGB4,
    Rgb5 = glow::RGB5,
    Rgb8 = glow::RGB8,
    Rgb8Snorm = glow::RGB8_SNORM,
    Rgb10 = glow::RGB10,
    Rgb12 = glow::RGB12,
    Rgb16Snorm = glow::RGB16_SNORM,
    Rgba2 = glow::RGBA2,
    Rgba4 = glow::RGBA4,
    Rgb5a1 = glow::RGB5_A1,
    Rgba8 = glow::RGBA8,
    Rgba8Snorm = glow::RGBA8_SNORM,
    Rgb10a2 = glow::RGB10_A2,
    Rgb10a2ui = glow::RGB10_A2UI,
    Rgba12 = glow::RGBA12,
    Rgba16 = glow::RGBA16,
    Srgb8 = glow::SRGB8,
    Srgb8Alpha8 = glow::SRGB8_ALPHA8,
    R16f = glow::R16F,
    Rg16f = glow::RG16F,
    Rgb16f = glow::RGB16F,
    Rgba16f = glow::RGBA16F,
    R32f = glow::R32F,
    Rg32f = glow::RG32F,
    Rgb32f = glow::RGB32F,
    Rgba32f = glow::RGBA32F,
    R11fg11fb10f = glow::R11F_G11F_B10F,
    Rgb9e5 = glow::RGB9_E5,
    R8i = glow::R8I,
    R8ui = glow::R8UI,
    R16i = glow::R16I,
    R16ui = glow::R16UI,
    R32i = glow::R32I,
    R32ui = glow::R32UI,
    Rg8i = glow::RG8I,
    Rg8ui = glow::RG8UI,
    Rg16i = glow::RG16I,
    Rg16ui = glow::RG16UI,
    Rg32i = glow::RG32I,
    Rg32ui = glow::RG32UI,
    Rgb8i = glow::RGB8I,
    Rgb8ui = glow::RGB8UI,
    Rgb16i = glow::RGB16I,
    Rgb16ui = glow::RGB16UI,
    Rgb32i = glow::RGB32I,
    Rgb32ui = glow::RGB32UI,
    Rgba8i = glow::RGBA8I,
    Rgba8ui = glow::RGBA8UI,
    Rgba16i = glow::RGBA16I,
    Rgba16ui = glow::RGBA16UI,
    Rgba32i = glow::RGBA32I,
    Rgba32ui= glow::RGBA32UI,
}

impl InternalFormat {
    pub fn raw(self) -> u32 {
        self as u32
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum Format {
    R = glow::RED,
    Rg = glow::RG,
    Rgb = glow::RGB,
    Bgr = glow::BGR,
    Rgba = glow::RGBA,
    Bgra = glow::BGRA,
    DepthComponent = glow::DEPTH_COMPONENT,
    StencilIndex = glow::STENCIL_INDEX
}

impl Format {
    pub fn raw(self) -> u32 {
        self as u32
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum FramebufferAttachment {
    Color(u32),
    Depth,
    Stencil,
    DepthStencil
}

impl FramebufferAttachment {
    pub fn raw(self) -> u32 {
        match self {
            FramebufferAttachment::Color(i) => glow::COLOR_ATTACHMENT0 + i,
            FramebufferAttachment::Depth => glow::DEPTH_ATTACHMENT,
            FramebufferAttachment::Stencil => glow::STENCIL_ATTACHMENT,
            FramebufferAttachment::DepthStencil => glow::DEPTH_STENCIL_ATTACHMENT
        }
    }
}
