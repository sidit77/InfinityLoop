
#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ShaderType {
    Vertex,
    Fragment,
    Geometry,
    TesselationControl,
    TesselationEvaluation,
    Compute
}

impl ShaderType {
    pub fn raw(self) -> u32 {
        match self {
            ShaderType::Vertex => glow::VERTEX_SHADER,
            ShaderType::Fragment => glow::FRAGMENT_SHADER,
            ShaderType::Geometry => glow::GEOMETRY_SHADER,
            ShaderType::TesselationControl => glow::TESS_CONTROL_SHADER,
            ShaderType::TesselationEvaluation => glow::TESS_EVALUATION_SHADER,
            ShaderType::Compute => glow::COMPUTE_SHADER
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PrimitiveType {
    Points,
    LineStrip,
    LineLoop,
    Lines,
    LineStripAdjacency,
    LinesAdjacency,
    TriangleStrip,
    TriangleFan,
    Triangles,
    TriangleStripAdjacency,
    TrianglesAdjacency,
    Patches
}

impl PrimitiveType {

    pub fn raw(self) -> u32 {
        match self {
            PrimitiveType::Points => glow::POINTS,
            PrimitiveType::LineStrip => glow::LINE_STRIP,
            PrimitiveType::LineLoop => glow::LINE_LOOP,
            PrimitiveType::Lines => glow::LINES,
            PrimitiveType::LineStripAdjacency => glow::LINE_STRIP_ADJACENCY,
            PrimitiveType::LinesAdjacency => glow::LINES_ADJACENCY,
            PrimitiveType::TriangleStrip => glow::TRIANGLE_STRIP,
            PrimitiveType::TriangleFan => glow::TRIANGLE_FAN,
            PrimitiveType::Triangles => glow::TRIANGLES,
            PrimitiveType::TriangleStripAdjacency => glow::TRIANGLE_STRIP_ADJACENCY,
            PrimitiveType::TrianglesAdjacency => glow::TRIANGLES_ADJACENCY,
            PrimitiveType::Patches => glow::PATCHES
        }
    }

}