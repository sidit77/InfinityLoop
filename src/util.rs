pub trait Update {
    fn update(&mut self, value: Self) -> bool;
}

impl<T: Eq> Update for T {
    fn update(&mut self, value: Self) -> bool {
        if *self == value {
            false
        } else {
            *self = value;
            true
        }
    }
}

pub trait OptionExt<T> {
    fn contains_e<U: PartialEq<T>>(&self, x: &U) -> bool;
}

impl<T> OptionExt<T> for Option<T> {
    fn contains_e<U: PartialEq<T>>(&self, x: &U) -> bool {
        match self {
            Some(y) => x.eq(y),
            None => false,
        }
    }
}

#[macro_export]
macro_rules! log_assert {
    ($($arg:tt)*) => (if !($($arg)*) { log::warn!("Assertion failed: {} at {}:{}", std::stringify!($($arg)*), std::file!(), std::line!()); })
}

#[macro_export]
macro_rules! log_unreachable {
    ($($arg:tt)*) => (log::warn!("Unreachable line was reached: {}:{}", std::file!(), std::line!()))
}