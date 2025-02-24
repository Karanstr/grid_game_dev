use macroquad::{
    shapes::{draw_circle, draw_circle_lines, draw_line, draw_rectangle, draw_rectangle_lines, draw_triangle, draw_triangle_lines},
    color::*,
    miniquad::window::screen_size,
};
use macroquad::math::Vec2;
use derive_new::new;
use crate::engine::math::Aabb;
#[derive(new)]
pub struct Camera { 
    position: Vec2,
    radius: f32,
    #[new(value = "1.")]
    scale: f32,
}
// State changes
impl Camera {
    pub fn update(&mut self, move_to:Option<(Vec2, f32)>) {
        if let Some((new_position, smoothing)) = move_to {
            self.lerp_position(new_position, smoothing);
        }
        self.update_scale();
    }

    pub fn update_scale(&mut self) {
        self.scale = (Vec2::from(screen_size())).min_element() / (2. * self.radius);
    }

    pub fn move_to(&mut self, new_position:Vec2, smoothing:f32) {
        self.lerp_position(new_position, smoothing);
    }

    pub fn change_zoom(&mut self, zoom:f32) {
        self.radius /= zoom;
    }

    fn lerp_position(&mut self, position:Vec2, smoothing:f32) {
        self.position = self.position.lerp(position, smoothing);
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

    pub fn show_view(&self) {
        self.outline_bounds(Aabb::new(self.position, Vec2::splat(self.radius)), 0.05, WHITE);
    }

    /*pub fn render_grid(&self, location:Location, rotation:Vec2, alpha:u8) {
        let point_offset = center_to_edge(location.pointer.height, location.min_cell_length);
        let points_list: Vec<([Vec2; 4], usize)> = self.corners.iter().map(|cell| {
            ([
                    (cell.points[0] - point_offset).rotate(rotation) + location.position,
                    (cell.points[1] - point_offset).rotate(rotation) + location.position,
                    (cell.points[2] - point_offset).rotate(rotation) + location.position,
                    (cell.points[3] - point_offset).rotate(rotation) + location.position
                ], *cell.index
            )
        }).collect();
        for (points, index) in points_list {
            let color = crate::globals::BLOCKS.color(index);
            if color == BLANK { continue; }
            self.draw_rectangle_from_corners(
                &points,
                Color::from_rgba((color.r * 255.) as u8, (color.g * 255.) as u8, (color.b * 255.) as u8, alpha),
                render_dbg,
            );
        }

    }*/

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
