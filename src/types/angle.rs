use std::cmp::Ordering;
use std::fmt::{Display, Formatter};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign};

#[derive(Debug, Default, Copy, Clone)]
pub struct Angle(f32);

#[allow(dead_code)]
impl Angle {

    pub fn empty() -> Self {
        Self::radians(0.0)
    }

    pub fn half() -> Self {
        Self::radians(std::f32::consts::PI)
    }

    pub fn full() -> Self {
        Self::radians(std::f32::consts::TAU)
    }

    pub fn radians(radians: f32) -> Self {
        Angle(radians)
    }

    pub fn degrees(degrees: f32) -> Self {
        Self::radians(degrees.to_radians())
    }

    pub fn to_radians(self) -> f32 {
        self.0
    }

    pub fn to_degrees(self) -> f32 {
        self.0.to_degrees()
    }

    pub fn normalized(mut self) -> Self {
        self.0 %= Self::full().0;
        self.0 += Self::full().0;
        self.0 %= Self::full().0;
        self
    }

    pub fn max(self, rhs: Self) -> Self {
        Angle(f32::max(self.normalized().0, rhs.normalized().0))
    }

    pub fn min(self, rhs: Self) -> Self {
        Angle(f32::min(self.normalized().0, rhs.normalized().0))
    }

    pub fn lerp(self, rhs: Self, factor: f32) -> Self {
        let mut diff = (rhs - self).normalized();
        if diff > Self::half() {
            diff -= Self::full()
        }
        self + diff * factor
    }

    pub fn lerp_snap(self, rhs: Self, factor: f32, threshold: Self) -> Self {
        let mut diff = (rhs - self).normalized();
        if diff > Self::half() {
            diff -= Self::full()
        }
        if diff.abs() < threshold {
            rhs
        } else {
            self + diff * factor
        }
    }

    pub fn abs(self) -> Self {
        Angle(self.0.abs())
    }

}

impl Add for Angle {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Angle(self.0 + rhs.0)
    }
}

impl AddAssign for Angle {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sub for Angle {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Angle(self.0 - rhs.0)
    }
}

impl SubAssign for Angle {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Mul<f32> for Angle {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Angle(self.0 * rhs)
    }
}

impl MulAssign<f32> for Angle {
    fn mul_assign(&mut self, rhs: f32) {
        *self = *self * rhs
    }
}

impl Div<f32> for Angle {
    type Output = Self;

    fn div(self, rhs: f32) -> Self::Output {
        Angle(self.0 / rhs)
    }
}

impl DivAssign<f32> for Angle {
    fn div_assign(&mut self, rhs: f32) {
        *self = *self / rhs
    }
}

impl Display for Angle {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}°", self.normalized().to_degrees())
    }
}

impl PartialEq<Self> for Angle {
    fn eq(&self, other: &Self) -> bool {
        self.normalized().0.eq(&other.normalized().0)
    }
}

impl Eq for Angle {}

impl PartialOrd<Self> for Angle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.normalized().0.partial_cmp(&other.normalized().0)
    }
}
