use macroquad::{
    shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines, draw_triangle, draw_triangle_lines},
    color::*,
    miniquad::window::screen_size,
};
use macroquad::math::Vec2;
pub struct Camera { 
    position: Vec2,
    radius: f32,
    scale: f32,
}
// State changes
impl Camera {
    pub fn new(position: Vec2, radius: f32) -> Self {
        Self {
            position,
            radius,
            scale: (Vec2::from(screen_size())).min_element() / (2. * radius),
        }
    }
    
    /// Moves the camera and fixes resolution to new screen size (I think)
    pub fn update(&mut self, move_to:Option<(Vec2, f32)>) {
        if let Some((new_position, smoothing)) = move_to {
            self.position = self.position.lerp(new_position, smoothing);
        }
        self.scale = (Vec2::from(screen_size())).min_element() / (2. * self.radius);
    }

    pub fn change_zoom(&mut self, zoom:f32) {
        self.radius /= zoom;
    }
}
// Conversions between screen and world spaces
impl Camera {
    fn global_offset(&self) -> Vec2 {
        self.position - Vec2::from(screen_size()) / 2. / self.scale
    }

    pub fn world_to_screen(&self, world_position:Vec2) -> Vec2 {
       (world_position - self.global_offset()) * self.scale
    }

    pub fn screen_to_world(&self, screen_position:Vec2) -> Vec2 {
        screen_position / self.scale + self.global_offset()
    }
}
// Drawing methods
impl Camera {

    pub fn draw_vec_rectangle(&self, position:Vec2, length:Vec2, color:Color) {
        let pos = self.world_to_screen(position);
        let len = length * self.scale;
        draw_rectangle(pos.x, pos.y, len.x, len.y, color);
    }

    pub fn outline_vec_rectangle(&self, position:Vec2, length:Vec2, line_width:f32, color:Color) {
        let pos = self.world_to_screen(position);
        let len = length * self.scale;
        draw_rectangle_lines(pos.x, pos.y, len.x, len.y, line_width*self.scale, color);
    }
    
    pub fn draw_point(&self, position:Vec2, radius:f32, color:Color) {
        let pos = self.world_to_screen(position);
        draw_circle(pos.x, pos.y, radius*self.scale, color);
    }

    pub fn outline_point(&self, position:Vec2, radius:f32, thickness:f32, color:Color) {
        let pos = self.world_to_screen(position);
        draw_circle_lines(pos.x, pos.y, radius*self.scale, thickness*self.scale, color);
    }

    pub fn draw_vec_line(&self, point1:Vec2, point2:Vec2, color:Color) {
        let p1 = self.world_to_screen(point1);
        let p2 = self.world_to_screen(point2);
        draw_line(p1.x, p1.y, p2.x, p2.y, 2., color);
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
