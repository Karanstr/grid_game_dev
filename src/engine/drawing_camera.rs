use macroquad::prelude::*;
use macroquad::miniquad::window::screen_size;
use crate::engine::utilities::{BoundingRect, AABB};

pub struct Camera { 
    pub aabb:AABB,
    pub offset:Vec2,
    pub zoom:f32,
}
#[allow(dead_code)]
impl Camera {
    pub fn new(aabb:AABB, offset:Vec2) -> Self {
        Self { aabb, offset, zoom : 1. }
    }

    pub fn show_view(&self) {
        self.outline_bounds(self.aabb, 2., WHITE);
    }

    pub fn interpolate_position(&mut self, position:Vec2, smoothing:f32) {
        self.aabb.move_to(self.aabb.center().lerp(position, smoothing));
    }

    pub fn interpolate_offset(&mut self, target:Vec2, smoothing:f32) {
        self.offset = self.offset.lerp(target, smoothing);
    }

    //This is a little scuffed bc of how I semantically designed the shrink and expand functions
    //I'll revisit this at some point
    pub fn expand_view(&mut self, distance:Vec2) {
        let center = self.aabb.center();
        self.aabb = self.aabb.expand(distance);
        self.aabb.move_to(center);
    }

    pub fn shrink_view(&mut self, distance:Vec2) {
        let center = self.aabb.center();
        self.aabb = self.aabb.shrink(distance);
        self.aabb.move_to(center);
    }

    pub fn camera_global_offset(&self) -> Vec2 {
        self.aabb.center() + self.offset - Vec2::from(screen_size())/2./self.zoom
    }

     pub fn draw_centered_square(&self, position:Vec2, length:f32, color:Color) {
        let real_pos = position - length/2. - self.camera_global_offset();
        draw_rectangle(real_pos.x * self.zoom, real_pos.y * self.zoom, length * self.zoom, length * self.zoom, color);
    }

    pub fn outline_centered_square(&self, position:Vec2, length:f32, line_width:f32, color:Color) {
        let real_pos = position - length/2. - self.camera_global_offset();
        draw_rectangle_lines(real_pos.x * self.zoom, real_pos.y * self.zoom, length * self.zoom, length * self.zoom, line_width, color);
    }

    pub fn draw_vec_rectangle(&self, position:Vec2, length:Vec2, color:Color) {
        let pos = position - self.camera_global_offset();
        draw_rectangle(pos.x * self.zoom, pos.y * self.zoom, length.x * self.zoom, length.y * self.zoom, color);
    }

    pub fn outline_vec_rectangle(&self, position:Vec2, length:Vec2, line_width:f32, color:Color) {
        let pos = position - self.camera_global_offset();
        draw_rectangle_lines(pos.x * self.zoom, pos.y * self.zoom, length.x * self.zoom, length.y * self.zoom, line_width, color);
    }
    
    pub fn draw_vec_circle(&self, position:Vec2, radius:f32, color:Color) {
        let pos = position - self.camera_global_offset();
        draw_circle(pos.x * self.zoom, pos.y * self.zoom, radius * self.zoom, color);
    }

    pub fn outline_vec_circle(&self, position:Vec2, radius:f32, line_width:f32, color:Color) {
        let pos = position - self.camera_global_offset();
        draw_circle_lines(pos.x * self.zoom, pos.y * self.zoom, radius * self.zoom, line_width, color);
    }

    pub fn draw_vec_line(&self, point1:Vec2, point2:Vec2, line_width:f32, color:Color) {
        let p1 = point1 - self.camera_global_offset();
        let p2 = point2 - self.camera_global_offset();
        draw_line(p1.x * self.zoom, p1.y * self.zoom, p2.x * self.zoom, p2.y * self.zoom, line_width, color);
    }

    pub fn draw_bounds<T:BoundingRect>(&self, bounds:T, color:Color) {
        self.draw_vec_rectangle(bounds.min(), bounds.max() - bounds.min(), color);
    }

    pub fn outline_bounds<T:BoundingRect>(&self, bounds:T, line_width:f32, color:Color) {
        self.outline_vec_rectangle(bounds.min(), bounds.max() - bounds.min(), line_width, color);
    }


}