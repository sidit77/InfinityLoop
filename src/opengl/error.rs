use std::error::Error;
use std::fmt::{Display, Formatter};
pub type GlResult<T> = Result<T, GlError>;

#[derive(Debug, Clone)]
pub struct GlError(String);

impl Display for GlError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl Error for GlError {}

impl From<String> for GlError {
    fn from(reason: String) -> Self {
        Self(reason)
    }
}

impl From<&str> for GlError {
    fn from(reason: &str) -> Self {
        Self(String::from(reason))
    }
}