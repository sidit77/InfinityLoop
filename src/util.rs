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