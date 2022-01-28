use std::fmt::{Debug, Formatter};
use std::iter::{once, repeat};
use std::ops::{Add, AddAssign, Mul, MulAssign, Sub, SubAssign};
use glam::{Mat2, Vec2, Vec3};

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct HexPos(i32, i32);

const NEIGHBOR_OFFSETS: &[HexPos] = &[
    HexPos(1, 0), HexPos(1, -1), HexPos(0, -1),
    HexPos(-1, 0), HexPos(-1, 1), HexPos(0, 1)
];

impl HexPos {

    pub const CENTER: Self = Self::new(0, 0);

    pub const fn new(q: i32, r: i32) -> Self {
        Self(q, r)
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

    pub fn neighbors(self) -> impl Iterator<Item=Self> {
        NEIGHBOR_OFFSETS.into_iter().map(move |offset| self + *offset)
    }

    pub fn ring_iter(center: Self, radius: i32) -> impl Iterator<Item=Self> {
        debug_assert!(radius >= 1);
        NEIGHBOR_OFFSETS
            .iter()
            .flat_map(move |d|
                repeat(*d).take(radius as usize))
            .scan(center + NEIGHBOR_OFFSETS[4] * radius, |state, step| {
                let hex = *state;
                *state += step;
                Some(hex)
            })
    }

    pub fn spiral_iter(center: Self, radius: i32) -> impl Iterator<Item=Self> {
        debug_assert!(radius >= 0);
        once(center)
            .chain((1..=radius)
                .flat_map(move |i| Self::ring_iter(center, i)))
    }

}

impl Debug for HexPos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("")
            .field(&self.q())
            .field(&self.r())
            .field(&self.s())
            .finish()
    }
}

//[f32::sqrt(3.0) / 3.0, 0.0, -1.0 / 3.0, 2.0 / 3.0]
const POINT_TO_HEX: &[f32; 4] = &[0.57735026, 0.0, -0.33333334, 0.6666667];

impl From<Vec2> for HexPos {
    fn from(pt: Vec2) -> Self {
        let pt = Mat2::from_cols_array(POINT_TO_HEX) * pt;
        let pt = cube_round(pt.extend(-pt.x-pt.y));
        HexPos(pt.x as i32, pt.y as i32)
    }
}

fn cube_round(frac: Vec3) -> Vec3 {
    let mut round = frac.round();
    let diff = (round - frac).abs();

    if diff.x > diff.y && diff.x > diff.z {
        round.x = -round.y - round.z
    } else if diff.y > diff.z {
        round.y = -round.x - round.z
    } else {
        round.z = -round.x - round.y
    }
    round
}

//[f32::sqrt(3.0), 0.0, f32::sqrt(3.0) / 2.0, 3.0 / 2.0]
const HEX_TO_POINT: &[f32; 4] = &[1.7320508, 0.0, 0.8660254, 1.5];

impl From<HexPos> for Vec2 {
    fn from(hex: HexPos) -> Self {
        Mat2::from_cols_array(HEX_TO_POINT) * Vec2::new(hex.q() as f32,hex.r() as f32)
    }
}

impl Add for HexPos {
    type Output = HexPos;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl AddAssign for HexPos {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs
    }
}

impl Sub for HexPos {
    type Output = HexPos;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0, self.1 - rhs.1)
    }
}

impl SubAssign for HexPos {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs
    }
}

impl Mul<i32> for HexPos {
    type Output = Self;

    fn mul(self, rhs: i32) -> Self::Output {
        Self(self.0 * rhs, self.1 * rhs)
    }
}

impl MulAssign<i32> for HexPos {
    fn mul_assign(&mut self, rhs: i32) {
        *self = *self * rhs
    }
}