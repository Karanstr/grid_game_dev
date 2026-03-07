use macroquad::{
    shapes::{draw_circle, draw_line, draw_triangle, draw_triangle_lines},
    color::*,
    miniquad::window::screen_size,
    math::Vec2 as MqVec2,
};
use glam::Vec2;
pub struct Camera { 
    position: Vec2,
    radius: f32,
    scale: f32,

    // Cache
    screen_size: Vec2,
    // Flags
    dbg: bool,
}

// State changes
impl Camera {

    pub fn new(position: Vec2, radius: f32) -> Self {
        let mut new = Self {
            position,
            radius,
            scale: 1.,

            screen_size: Vec2::from(screen_size()),
            dbg: true,
        };
        new.fix_scale();
     
        new
    }
    
    pub fn follow(&mut self, target: Vec2, smoothing: f32) {
        self.position = self.position.lerp(target, smoothing)
    }
    
    pub fn fix_scale(&mut self) {
        self.scale = self.screen_size.min_element() / (2. * self.radius);
    }

    pub fn update_screensize(&mut self) {
        let new_screen_size = Vec2::from(screen_size());
        if new_screen_size != self.screen_size {
            self.screen_size = new_screen_size;
            self.fix_scale()
        }
    }

    /// Takes a zoom factor, so zoom_by(1.5) zooms in by 50%
    pub fn zoom_by(&mut self, zoom: f32) { 
        self.radius /= zoom;
        self.fix_scale();
    }

}

// Conversions between screen and world spaces
impl Camera {
    
    /// Offset from top left of the screen to center
    fn screen_offset(&self) -> Vec2 { Vec2::from(screen_size()) / 2. }

    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        (world_pos - self.position) * self.scale + self.screen_offset()
    }

    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        (screen_pos - self.screen_offset()) / self.scale + self.position
    }

}

// Drawing methods
impl Camera {

    pub fn draw_point(&self, position: Vec2, radius: f32, color: Color) {
        let pos = self.world_to_screen(position);
        draw_circle(pos.x, pos.y, radius * self.scale, color);
    }

    pub fn draw_vec_line(&self, point1: Vec2, point2: Vec2, thickness: f32, color: Color) {
        let p1 = self.world_to_screen(point1);
        let p2 = self.world_to_screen(point2);
        draw_line(p1.x, p1.y, p2.x, p2.y, thickness, color);
    }

    pub fn draw_rectangle_from_corners(&self, corners:&[Vec2; 4], color: Color) {
        let corners: Vec<MqVec2> = corners.iter().map(|point| {
            let corner = self.world_to_screen(*point);
            MqVec2::new(corner.x, corner.y)
        }).collect();
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

        if self.dbg {
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

    pub fn draw_outline(&self, points:&[Vec2], thickness: f32, color: Color) {
        let points:Vec<Vec2> = points.iter().map(|point| self.world_to_screen(*point)).collect();
        for point in 0 .. points.len() {
            let point1 = points[point];
            let point2 = points[(point + 1) % points.len()];
            draw_line(point1.x, point1.y, point2.x, point2.y, thickness, color);
        }
    }

}
