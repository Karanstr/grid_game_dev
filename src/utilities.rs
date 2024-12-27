use macroquad::prelude::*;
use crate::drawing_camera::Camera;

#[allow(dead_code)]
pub trait BoundingRect {
    fn min(&self) -> Vec2;
    fn max(&self) -> Vec2;
    fn center(&self) -> Vec2;
    fn intersects(&self, other:Self) -> BVec2;
    fn contains(&self, point:Vec2) -> BVec2;
    fn move_by(&mut self, displacement:Vec2);
    fn move_to(&mut self, position:Vec2);
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
    fn intersects(&self, other:Self) -> BVec2 {
        let offset = (other.center - self.center).abs();
        BVec2::new(
            offset.x < self.radius.x + other.radius.x,
            offset.y < self.radius.y + other.radius.y,
        )
    }
    fn contains(&self, point:Vec2) -> BVec2 {
        let offset = (point - self.center).abs();
        BVec2::new(
            offset.x < self.radius.x,
            offset.y < self.radius.y,
        )
    }
    fn move_by(&mut self, displacement:Vec2) { self.center += displacement }
    fn move_to(&mut self, position:Vec2) { self.center = position }
}
impl AABB {
    pub fn new(center:Vec2, radius:Vec2) -> Self { Self { center, radius } }
    pub fn radius(&self) -> Vec2 { self.radius }
    pub fn set_radius(&mut self, radius:Vec2) { self.radius = radius }

    pub fn expand(&self, distance:Vec2) -> Self {
        Self {
            center: self.center + distance / 2.,
            radius: self.radius + distance.abs() / 2.,
        }
    }
    pub fn shrink(&self, distance:Vec2) -> Self {
        Self {
            center: self.center - distance / 2.,
            radius: (self.radius - distance.abs() / 2.).abs(),
        }
    }
}

#[allow(dead_code)]
pub trait Vec2Extension {
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



pub struct Drawable {
    pub position : Vec2,
    pub color : Color,
    pub ticks_remaining : i32,
}
pub struct DebugRender {
    pub drawables: Vec<Drawable>,
}
impl DebugRender {
    pub fn new() -> Self { 
        Self { drawables: Vec::new() } 
    }
    pub fn add(&mut self, position:Vec2, color:Color, ticks_remaining:i32) { 
        self.drawables.push(Drawable { position, color, ticks_remaining });
    }
    pub fn draw(&mut self, camera:&Camera) { 
        for drawable in self.drawables.iter_mut() { 
            drawable.ticks_remaining -= 1;
            camera.draw_centered_square(drawable.position, 5., drawable.color);
        } 
        self.drawables.retain(|drawable| drawable.ticks_remaining > 0);
    }
}