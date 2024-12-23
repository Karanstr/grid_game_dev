use macroquad::prelude::*;

pub trait BoundingRect {
    fn min(&self) -> Vec2;
    fn max(&self) -> Vec2;
    fn center(&self) -> Vec2;
    fn intersects(&self, other:Self) -> bool;
    fn move_by(&mut self, displacement:Vec2);
}
#[derive(Clone, Copy, Debug)]
pub struct AABB {
    center: Vec2,
    radius: Vec2,
}
impl BoundingRect for AABB {
    fn min(&self) -> Vec2 { self.center - self.radius }
    fn max(&self) -> Vec2 { self.center + self.radius }
    fn center(&self) -> Vec2 { self.center }
    fn intersects(&self, other:Self) -> bool {
        let offset = (other.center - self.center).abs();
        offset.x < self.radius.x + other.radius.x && offset.y < self.radius.y + other.radius.y
    }
    fn move_by(&mut self, displacement:Vec2) { self.center += displacement }
}
impl AABB {
    pub fn new(center:Vec2, radius:Vec2) -> Self { Self { center, radius } }
    pub fn radius(&self) -> Vec2 { self.radius }
    pub fn extend(&self, distance:Vec2) -> Self {
        Self {
            center: self.center + distance / 2.,
            radius: self.radius + distance.abs() / 2.,
        }
    }
}

#[allow(dead_code)]
trait Vec2Extension {
    fn better_sign(&self) -> Vec2; 
}
impl Vec2Extension for Vec2 {
    fn better_sign(&self) -> Vec2 {
        Vec2::new(
            if self.x < 0. { -1. } else if self.x > 0. { 1. } else { 0. },
            if self.y < 0. { -1. } else if self.y > 0. { 1. } else { 0. },
        )
    }
}