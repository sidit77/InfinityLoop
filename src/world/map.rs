use std::fmt::{Debug, Formatter};
use crate::HexPos;

#[derive(Clone)]
pub struct HexMap<T> {
    radius: i32,
    elements: Box<[T]>
}

impl<T> AsRef<[T]> for HexMap<T> {
    fn as_ref(&self) -> &[T] {
        &self.elements
    }
}

impl<T: Debug> Debug for HexMap<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.keys().map(|k| (k, self.get(k).unwrap()))).finish()
    }
}

impl<T: Default + Clone> HexMap<T> {
    pub fn new(radius: i32) -> Self {
        debug_assert!(radius >= 0);
        let diameter = 2 * radius + 1;
        let len = diameter * diameter - radius * radius - radius;
        let elements = vec![Default::default(); len as usize].into_boxed_slice();
        Self {
            radius,
            elements
        }
    }

    pub fn fill(&mut self, value: T) {
        self.elements.fill(value)
    }
}

impl<T> HexMap<T> {

    pub fn from<U>(old: &HexMap<U>, func: impl Fn(&U) -> T) -> Self {
        Self {
            radius: old.radius,
            elements: old.elements.iter().map(func).collect()
        }
    }

    pub fn len(&self) -> usize {
        self.elements.len()
    }

    pub fn keys(&self) -> impl Iterator<Item=HexPos> {
        HexPos::spiral_iter(self.center(), self.radius)
    }

    pub fn values(&self) -> impl Iterator<Item=&T> {
        self.elements.iter()
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item=&mut T> {
        self.elements.iter_mut()
    }

    pub fn radius(&self) -> i32 {
        self.radius
    }

    pub fn center(&self) -> HexPos {
        HexPos::CENTER
    }

    pub fn index(&self, pos: HexPos) -> Option<usize> {
        if !self.contains(pos) {
            None
        } else {
            let diameter = 2 * self.radius + 1;
            let len = self.elements.len() as i32;
            let sign = 1 | (pos.q() >> 31);
            let r = sign * pos.r();
            let q = sign * pos.q();
            let index = len / 2 + sign * (r + q * diameter - ((q - 1) * q) / 2);
            Some(index as usize)
        }
    }

    pub fn contains(&self, pos: HexPos) -> bool {
        pos.q().abs() <= self.radius && pos.r().abs() <= self.radius && pos.s().abs() <= self.radius
    }

    pub fn get(&self, pos: HexPos) -> Option<&T> {
        self.index(pos).map(|i|&self.elements[i])
    }

    pub fn get_mut(&mut self, pos: HexPos) -> Option<&mut T> {
        self.index(pos).map(move |i|&mut self.elements[i])
    }

}
