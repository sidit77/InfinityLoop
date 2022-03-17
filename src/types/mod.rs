mod angle;
mod color;
mod hex;

pub use angle::Angle;
pub use color::Rgba;
pub use hex::HexPos;

pub type Color = Rgba<u8>;