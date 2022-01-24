mod angle;
mod color;
mod hex;

pub use angle::Angle;
pub use color::RGBA;
pub use hex::HexPosition;

pub type Color = RGBA<u8>;