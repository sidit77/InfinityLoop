use std::ops::{Deref, DerefMut};
use glam::Vec2;
use crate::{Angle, Camera};

pub trait Lerp: Copy {
    fn lerp(self, other: Self, v: f32) -> Self;
}

impl Lerp for f32 {
    fn lerp(self, other: Self, v: f32) -> Self {
        self + (other - self) * v
    }
}

impl Lerp for Angle {
    fn lerp(self, other: Self, v: f32) -> Self {
        self.lerp(other, v)
    }
}

impl Lerp for Vec2 {
    fn lerp(self, other: Self, v: f32) -> Self {
        self.lerp(other, v)
    }
}

impl Lerp for Camera {
    fn lerp(mut self, other: Self, v: f32) -> Self {
        self.position = Lerp::lerp(self.position, other.position, v);
        self.scale = Lerp::lerp(self.scale, other.scale, v);
        self.aspect = Lerp::lerp(self.aspect, other.aspect, v);
        self.rotation = Lerp::lerp(self.rotation, other.rotation, v);
        self
    }
}

pub trait Diff {
    fn update_on_diff(&mut self, old: &Self, new: &Self);
}

impl Diff for f32 {
    fn update_on_diff(&mut self, old: &Self, new: &Self) {
        if old.ne(new) {
            *self = *new;
        }
    }
}

impl Diff for Vec2 {
    fn update_on_diff(&mut self, old: &Self, new: &Self) {
        self.x.update_on_diff(&old.x, &new.x);
        self.y.update_on_diff(&old.y, &new.y);
    }
}

impl Diff for Angle {
    fn update_on_diff(&mut self, old: &Self, new: &Self) {
        if old.ne(new) {
            *self = *new;
        }
    }
}

impl Diff for Camera {
    fn update_on_diff(&mut self, old: &Self, new: &Self) {
        self.position.update_on_diff(&old.position, &new.position);
        self.aspect.update_on_diff(&old.aspect, &new.aspect);
        self.scale.update_on_diff(&old.scale, &new.scale);
        self.rotation.update_on_diff(&old.rotation, &new.rotation);
    }
}

#[derive(Debug, Copy, Clone)]
pub struct AnimatedValue<T: Copy> {
    current: T,
    target: T
}

impl<T: Copy> AnimatedValue<T> {
    pub fn new(val: T) -> Self {
        Self {
            current: val,
            target: val
        }
    }

    pub fn target(&mut self) -> &mut T {
        &mut self.target
    }

    #[allow(dead_code)]
    pub fn skip(&mut self) {
        self.current = self.target
    }

}

impl <T: Diff + Copy> AnimatedValue<T> {
    pub fn current(&mut self) -> RefMut<T> {
        RefMut::new(self)
    }
}

pub struct RefMut<'a, T: Copy + Diff>{
    value: &'a mut AnimatedValue<T>,
    old: T
}

impl<'a, T: Copy + Diff> RefMut<'a, T> {
    fn new(value: &'a mut AnimatedValue<T>) -> Self{
        let old = value.current;
        Self {
            value,
            old
        }
    }
}

impl<'a, T: Copy + Diff> Deref for RefMut<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.value.current
    }
}

impl<'a, T: Copy + Diff> DerefMut for RefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.value.current
    }
}

impl<'a, T: Copy + Diff> Drop for RefMut<'a, T> {
    fn drop(&mut self) {
        self.value.target.update_on_diff(&self.old, &self.value.current)
    }
}

impl<T: Lerp> AnimatedValue<T> {
    pub fn animate_lerp_exp(&mut self, v: f32){
        self.current = T::lerp(self.current, self.target, 1.0 - f32::exp(-v))
    }
}

impl<T: Copy> Deref for AnimatedValue<T>{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.current
    }
}
