#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(C)]
pub struct RGBA<T: Copy> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T
}

#[allow(dead_code)]
impl<T: Copy> RGBA<T> {

    pub fn new(r: T, g: T, b: T, a: T) -> Self {
        Self { r, g, b, a }
    }

    pub fn with_red(mut self, red: T) -> Self {
        self.r = red;
        self
    }

    pub fn with_green(mut self, green: T) -> Self {
        self.g = green;
        self
    }

    pub fn with_blue(mut self, blue: T) -> Self {
        self.b = blue;
        self
    }

    pub fn with_alpha(mut self, alpha: T) -> Self {
        self.a = alpha;
        self
    }

}

impl Default for RGBA<f32> {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }
}

impl Default for RGBA<u8> {
    fn default() -> Self {
        Self::new(0, 0, 0, 255)
    }
}


impl From<RGBA<f32>> for RGBA<u8>{
    fn from(color: RGBA<f32>) -> Self {
        const U8_MAX: f32 = u8::MAX as f32;
        RGBA::default()
            .with_red((U8_MAX * color.r.clamp(0.0, 1.0)) as u8)
            .with_green((U8_MAX * color.g.clamp(0.0, 1.0)) as u8)
            .with_blue((U8_MAX * color.b.clamp(0.0, 1.0)) as u8)
            .with_alpha((U8_MAX * color.a.clamp(0.0, 1.0)) as u8)
    }
}

impl From<RGBA<u8>> for RGBA<f32>{
    fn from(color: RGBA<u8>) -> Self {
        const U8_MAX: f32 = u8::MAX as f32;
        RGBA::default()
            .with_red(color.r as f32 / U8_MAX)
            .with_green(color.g as f32 / U8_MAX)
            .with_blue(color.b as f32 / U8_MAX)
            .with_alpha(color.a as f32 / U8_MAX)
    }
}