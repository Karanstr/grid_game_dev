use macroquad::shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines, draw_triangle, draw_triangle_lines};
pub use macroquad::color::colors::*;
pub use macroquad::color::Color;
use macroquad::miniquad::window::screen_size;
use super::*;
pub struct Camera { 
    pub aabb : Aabb,
    scale_zoom: f32,
    zoom:f32,
    screen_percentage: f32,
}
#[allow(dead_code)]
// Basic State changes
impl Camera {
    pub fn new(aabb:Aabb, screen_percentage:f32) -> Self {
        let scale_zoom = (Vec2::from(screen_size()) * screen_percentage).min_element() / (2. * aabb.radius().min_element());
        Self { 
            aabb, 
            scale_zoom,
            zoom: 1.,
            screen_percentage
        }
    }

    pub fn update(&mut self, new_position:Vec2, smoothing:f32) {
        self.lerp_position(new_position, smoothing);
        self.scale_zoom = (Vec2::from(screen_size())*self.screen_percentage).min_element() / (2. * self.aabb.radius().min_element());
    }

    pub fn change_zoom(&mut self, zoom:f32) { self.zoom *= zoom }

    pub fn change_screen_percentage(&mut self, screen_percentage:f32) {
        self.screen_percentage = screen_percentage;
        self.update(self.aabb.center(), 0.);
    }

    fn zoom(&self) -> f32 { self.zoom * self.scale_zoom }

    pub fn show_view(&self) {
        self.outline_bounds(self.aabb, 0.05, WHITE);
    }

    fn lerp_position(&mut self, position:Vec2, smoothing:f32) {
        self.aabb.move_to(self.aabb.center().lerp(position, smoothing));
    }
 
}
// Conversions between screen and world spaces
impl Camera {
    fn global_offset(&self) -> Vec2 {
        self.aabb.center() - Vec2::from(screen_size()) / 2. / self.zoom()
    }

    pub fn world_to_screen(&self, world_position:Vec2) -> Vec2 {
       (world_position - self.global_offset()) * self.zoom()
    }

    pub fn screen_to_world(&self, screen_position:Vec2) -> Vec2 {
        screen_position / self.zoom() + self.global_offset()
    }
}
// Drawing methods
impl Camera {
    #[allow(dead_code)]
    pub fn draw_vec_rectangle(&self, position:Vec2, length:Vec2, color:Color) {
        let pos = self.world_to_screen(position);
        let len = length * self.zoom();
        draw_rectangle(pos.x, pos.y, len.x, len.y, color);
    }

    pub fn outline_vec_rectangle(&self, position:Vec2, length:Vec2, line_width:f32, color:Color) {
        let pos = self.world_to_screen(position);
        let len = length * self.zoom();
        draw_rectangle_lines(pos.x, pos.y, len.x, len.y, line_width*self.zoom(), color);
    }
    
    pub fn draw_point(&self, position:Vec2, radius:f32, color:Color) {
        let pos = self.world_to_screen(position);
        draw_circle(pos.x, pos.y, radius*self.zoom(), color);
    }

    pub fn outline_point(&self, position:Vec2, radius:f32, thickness:f32, color:Color) {
        let pos = self.world_to_screen(position);
        draw_circle_lines(pos.x, pos.y, radius*self.zoom(), thickness*self.zoom(), color);
    }


    #[allow(dead_code)]
    pub fn draw_vec_line(&self, point1:Vec2, point2:Vec2, color:Color) {
        let p1 = self.world_to_screen(point1);
        let p2 = self.world_to_screen(point2);
        draw_line(p1.x, p1.y, p2.x, p2.y, 2., color);
    }

    pub fn outline_bounds(&self, bounds:Aabb, line_width:f32, color:Color) {
        self.outline_vec_rectangle(bounds.min(), bounds.max() - bounds.min(), line_width, color);
    } 

    pub fn draw_rectangle_from_corners(&self, corners:&[Vec2], color: Color, render_dbg:bool) {
        let corners:Vec<Vec2> = corners.iter().map(|point| self.world_to_screen(*point)).collect();
        draw_triangle(
            corners[0],
            corners[1],
            corners[2],
            color
        );
        draw_triangle(
            corners[1],
            corners[2],
            corners[3],
            color
        );
        if render_dbg {
            draw_triangle_lines(
                corners[0],
                corners[1],
                corners[2],
                2.,
                WHITE
            );
            draw_triangle_lines(
                corners[1],
                corners[2],
                corners[3],
                2.,
                WHITE
            );
        }
    }

    pub fn draw_outline(&self, points:&[Vec2], color:Color) {
        let points:Vec<Vec2> = points.iter().map(|point| self.world_to_screen(*point)).collect();
        for point in 0 .. points.len() {
            let point1 = points[point];
            let point2 = points[(point + 1) % points.len()];
            draw_line(point1.x, point1.y, point2.x, point2.y, 4., color);
        }
    }
}
