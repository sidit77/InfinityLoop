use std::fmt::{Debug, Formatter};
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct HexPosition(i32, i32);

impl HexPosition {

    pub const CENTER: Self = Self::new(0, 0);

    pub const fn new(q: i32, r: i32) -> Self {
        HexPosition(q, r)
    }

    pub const fn q(self) -> i32 {
        self.0
    }

    pub const fn r(self) -> i32 {
        self.1
    }

    pub const fn s(self) -> i32 {
        -self.q() - self.r()
    }

}

impl Debug for HexPosition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("")
            .field(&self.q())
            .field(&self.r())
            .field(&self.s())
            .finish()
    }
}

impl Add for HexPosition {
    type Output = HexPosition;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl AddAssign for HexPosition {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sub for HexPosition {
    type Output = HexPosition;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl SubAssign for HexPosition {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Mul<i32> for HexPosition {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs)
    }
}

impl MulAssign<i32> for HexPosition {
    fn mul_assign(&mut self, rhs: i32) {
        *self = *self * rhs
    }
}