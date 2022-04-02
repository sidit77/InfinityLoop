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

#[macro_export]
macro_rules! log_assert {
    ($($arg:tt)*) => (if !($($arg)*) { log::warn!("Assertion failed: {} at {}:{}", std::stringify!($($arg)*), std::file!(), std::line!()); })
}

#[macro_export]
macro_rules! log_unreachable {
    ($($arg:tt)*) => (log::warn!("Unreachable line was reached: {}:{}", std::file!(), std::line!()))
}