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

pub trait Apply: Sized {
    fn apply<F: FnOnce(&mut Self)>(mut self, func: F) -> Self {
        func(&mut self);
        self
    }
}

impl<T> Apply for T {}