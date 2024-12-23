use macroquad::prelude::*;
use crate::utilities::BoundingRect;

pub struct Camera { 
    position:Vec2,
    offset:Vec2,
    view_size:Vec2,
}
#[allow(dead_code)]
impl Camera {
    pub fn new(position:Vec2, offset:Vec2, view_size:Vec2) -> Self {
        Self { position, offset, view_size }
    }

    pub fn update(&mut self, position:Vec2, screen_size:Vec2) {
        self.position = position;
        self.view_size = screen_size;
    }

    pub fn interpolate_offset(&mut self, target: Vec2, smoothing: f32) {
        self.offset = self.offset.lerp(target, smoothing);
    }

    pub fn camera_global_offset(&self) -> Vec2 {
        self.position - self.view_size/2. + self.offset
    }

     pub fn draw_centered_square(&self, position:Vec2, length:f32, color:Color) {
        let real_pos = position - length/2. - self.camera_global_offset();
        draw_rectangle(real_pos.x, real_pos.y, length, length, color);
    }

    pub fn outline_centered_square(&self, position:Vec2, length:f32, line_width:f32, color:Color) {
        let real_pos = position - length/2. - self.camera_global_offset();
        draw_rectangle_lines(real_pos.x, real_pos.y, length, length, line_width, color);
    }

    pub fn draw_vec_rectangle(&self, position:Vec2, length:Vec2, color:Color) {
        let pos = position - self.camera_global_offset();
        draw_rectangle(pos.x, pos.y, length.x, length.y, color);
    }

    pub fn outline_vec_rectangle(&self, position:Vec2, length:Vec2, line_width:f32, color:Color) {
        let pos = position - self.camera_global_offset();
        draw_rectangle_lines(pos.x, pos.y, length.x, length.y, line_width, color);
    }
    
    pub fn draw_vec_circle(&self, position:Vec2, radius:f32, color:Color) {
        let pos = position - self.camera_global_offset();
        draw_circle(pos.x, pos.y, radius, color);
    }

    pub fn outline_vec_circle(&self, position:Vec2, radius:f32, line_width:f32, color:Color) {
        let pos = position - self.camera_global_offset();
        draw_circle_lines(pos.x, pos.y, radius, line_width, color);
    }

    pub fn draw_vec_line(&self, point1:Vec2, point2:Vec2, line_width:f32, color:Color) {
        let p1 = point1 - self.camera_global_offset();
        let p2 = point2 - self.camera_global_offset();
        draw_line(p1.x, p1.y, p2.x, p2.y, line_width, color);
    }

    pub fn draw_bounds<T:BoundingRect>(&self, bounds:T, color:Color) {
        self.draw_vec_rectangle(bounds.min(), bounds.max() - bounds.min(), color);
    }

    pub fn outline_bounds<T:BoundingRect>(&self, bounds:T, line_width:f32, color:Color) {
        self.outline_vec_rectangle(bounds.min(), bounds.max() - bounds.min(), line_width, color);
    }


}