#[derive(Debug, Clone, Copy, Eq, PartialEq)]
#[repr(transparent)]
pub struct Rgba<T: Copy>([T; 4]);

#[allow(dead_code)]
impl<T: Copy> Rgba<T> {

    pub fn new(r: T, g: T, b: T, a: T) -> Self {
        Self([r, g, b, a])
    }

    pub fn red(self) -> T {
        self.0[0]
    }

    pub fn green(self) -> T {
        self.0[1]
    }

    pub fn blue(self) -> T {
        self.0[2]
    }

    pub fn alpha(self) -> T {
        self.0[3]
    }

    pub fn with_red(mut self, red: T) -> Self {
        self.0[0] = red;
        self
    }

    pub fn with_green(mut self, green: T) -> Self {
        self.0[1] = green;
        self
    }

    pub fn with_blue(mut self, blue: T) -> Self {
        self.0[2] = blue;
        self
    }

    pub fn with_alpha(mut self, alpha: T) -> Self {
        self.0[3] = alpha;
        self
    }

}

impl<T: Copy> AsRef<[T]> for Rgba<T> {
    fn as_ref(&self) -> &[T] {
        &self.0
    }
}

impl Default for Rgba<f32> {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 1.0)
    }
}

impl Default for Rgba<u8> {
    fn default() -> Self {
        Self::new(0, 0, 0, 255)
    }
}

impl From<Rgba<f32>> for Rgba<u8>{
    fn from(color: Rgba<f32>) -> Self {
        const U8_MAX: f32 = u8::MAX as f32;
        Rgba::default()
            .with_red((U8_MAX * color.red().clamp(0.0, 1.0)) as u8)
            .with_green((U8_MAX * color.green().clamp(0.0, 1.0)) as u8)
            .with_blue((U8_MAX * color.blue().clamp(0.0, 1.0)) as u8)
            .with_alpha((U8_MAX * color.alpha().clamp(0.0, 1.0)) as u8)
    }
}

impl From<Rgba<u8>> for Rgba<f32>{
    fn from(color: Rgba<u8>) -> Self {
        const U8_MAX: f32 = u8::MAX as f32;
        Rgba::default()
            .with_red(color.red() as f32 / U8_MAX)
            .with_green(color.green() as f32 / U8_MAX)
            .with_blue(color.blue() as f32 / U8_MAX)
            .with_alpha(color.alpha() as f32 / U8_MAX)
    }
}
