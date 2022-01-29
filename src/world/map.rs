use std::fmt::{Debug, Formatter};
use crate::HexPos;

#[derive(Clone)]
pub struct HexMap<T> {
    radius: i32,
    elements: Box<[(HexPos, T)]>
}

impl<T: Debug> Debug for HexMap<T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_map().entries(self.key_values()).finish()
    }
}

fn linearize(pos: HexPos, radius: i32) -> usize {
    let diameter = 2 * radius + 1;
    let len = diameter * diameter - radius * radius - radius;
    let sign = 1 | (pos.q() >> 31);
    let r = sign * pos.r();
    let q = sign * pos.q();
    let index = len / 2 + sign * (r + q * diameter - ((q - 1) * q) / 2);
    index as usize
}

impl<T: Default + Clone> HexMap<T> {
    pub fn new(radius: i32) -> Self {
        debug_assert!(radius >= 0);
        let diameter = 2 * radius + 1;
        let len = diameter * diameter - radius * radius - radius;
        let mut elements = vec![(HexPos::CENTER, Default::default()); len as usize].into_boxed_slice();
        for pos in HexPos::spiral_iter(HexPos::CENTER, radius) {
            elements[linearize(pos, radius)].0 = pos;
        }
        Self {
            radius,
            elements
        }
    }

    pub fn fill(&mut self, value: T) {
        for (_, v) in self.elements.iter_mut() {
            *v = value.clone();
        }
    }

}

impl<T> HexMap<T> {
    pub fn from<U>(old: HexMap<U>, func: impl Fn(&U) -> T) -> Self {
        Self {
            radius: old.radius,
            elements: old.elements.iter().map(|(k, v)|(*k, func(v))).collect()
        }
    }
}

impl<T> HexMap<T> {

    pub fn keys(&self) -> impl Iterator<Item=HexPos> +'_ {
        self.elements.iter().map(|t|t.0)
    }

    pub fn values(&self) -> impl Iterator<Item=&T> {
        self.elements.iter().map(|t|&t.1)
    }

    pub fn key_values(&self) -> impl Iterator<Item=(HexPos, &T)>{
        self.elements.iter().map(|(k, v)|(*k, v))
    }

    pub fn radius(&self) -> i32 {
        self.radius
    }

    pub fn center(&self) -> HexPos {
        HexPos::CENTER
    }

    fn index(&self, pos: HexPos) -> Option<usize> {
        if !self.contains(pos) {
            None
        } else {
            Some(linearize(pos, self.radius))
        }
    }

    pub fn contains(&self, pos: HexPos) -> bool {
        pos.q().abs() <= self.radius && pos.r().abs() <= self.radius && pos.s().abs() <= self.radius
    }

    pub fn get(&self, pos: HexPos) -> Option<&T> {
        self.index(pos).map(|i|&self.elements[i].1)
    }

    pub fn get_mut(&mut self, pos: HexPos) -> Option<&mut T> {
        self.index(pos).map(move |i|&mut self.elements[i].1)
    }

    pub fn set(&mut self, pos: HexPos, value: T) -> bool {
        match self.index(pos) {
            Some(i) => {
                self.elements[i].1 = value;
                true
            }
            None => false
        }
    }

}
