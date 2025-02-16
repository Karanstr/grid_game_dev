use super::*;
use crate::globals::*;

impl EntityPool {
    pub fn draw_all(&self, outline:bool) {
        for entity in self.entities.iter() {
            let location = &entity.location;
            if CAMERA.read().aabb.intersects(location.to_aabb()) == BVec2::TRUE {
                entity.draw(outline);
                entity.draw_velocity_arrow(macroquad::color::DARKBLUE);
            }
        }
    }

}

impl Entity {
    pub fn draw_velocity_arrow(&self, color: macroquad::color::Color) {
        if self.velocity.is_zero() { return; }
        let start_pos = self.location.position;
        let direction = self.velocity.normalize();
        let arrow_length = self.location.min_cell_length * self.velocity.length() * 10.;
        let end_pos = start_pos + direction * arrow_length;
        
        CAMERA.read().draw_vec_line(start_pos, end_pos, color);
    }

    pub fn draw(&self, render_dbg:bool) {
        let point_offset = center_to_edge(self.location.pointer.height, self.location.min_cell_length);
        let points_list: Vec<([Vec2; 4], usize)> = self.corners.iter().filter_map(|cell| {
            if !render_dbg { None } else { Some(([
                    (cell.points[0] - point_offset).rotate(self.forward) + self.location.position,
                    (cell.points[1] - point_offset).rotate(self.forward) + self.location.position,
                    (cell.points[2] - point_offset).rotate(self.forward) + self.location.position,
                    (cell.points[3] - point_offset).rotate(self.forward) + self.location.position
                ], *cell.index
            ))}
        }).collect();
        for (points, index) in points_list {
            CAMERA.read().draw_rectangle_from_corners(
                &points,
                BLOCKS.color(index),
                render_dbg,
            );
        }
    }
    
    pub fn draw_outline(&self, color:macroquad::color::Color) {
        let point_offset = center_to_edge(self.location.pointer.height, self.location.min_cell_length);
        let square = ExternalPointer::new(Index(1), self.location.pointer.height);
        let corners = tree_corners(square, self.location.min_cell_length)[0].points;
        let points = [
            (corners[0] - point_offset).rotate(self.forward) + self.location.position,
            (corners[1] - point_offset).rotate(self.forward) + self.location.position,
            (corners[3] - point_offset).rotate(self.forward) + self.location.position,
            (corners[2] - point_offset).rotate(self.forward) + self.location.position,
        ];
        CAMERA.read().draw_outline(&points, color);
    }

}
