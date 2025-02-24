use super::*;
use crate::globals::*;
use crate::engine::grid::dag::Index;
impl EntityPool {
    pub fn draw_all(&self, rotate:bool, render_dbg:bool) {
        for entity in self.entities.iter() {
            entity.draw(rotate, render_dbg);
            entity.draw_velocity_arrow(macroquad::color::DARKBLUE);
        }
    }

}

impl Entity {
    pub fn draw_velocity_arrow(&self, color: macroquad::color::Color) {
        CAMERA.read().draw_vec_line(
            self.location.position, 
            self.location.position + self.velocity * 5.,
            color
        );
    }

    pub fn draw(&self, rotate:bool, render_dbg:bool) {
        let point_offset = center_to_edge(self.location.pointer.height, self.location.min_cell_length);
        let rotation = if rotate { self.forward } else { Vec2::new(1., 0.) };
        let points_list: Vec<([Vec2; 4], usize)> = self.corners.iter().map(|cell| {
            ([
                    (cell.points[0] - point_offset).rotate(rotation) + self.location.position,
                    (cell.points[1] - point_offset).rotate(rotation) + self.location.position,
                    (cell.points[2] - point_offset).rotate(rotation) + self.location.position,
                    (cell.points[3] - point_offset).rotate(rotation) + self.location.position
                ], *cell.index
            )
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
        let corners = corner_handling::tree_corners(square, self.location.min_cell_length)[0].points;
        let points = [
            (corners[0] - point_offset).rotate(self.forward) + self.location.position,
            (corners[1] - point_offset).rotate(self.forward) + self.location.position,
            (corners[3] - point_offset).rotate(self.forward) + self.location.position,
            (corners[2] - point_offset).rotate(self.forward) + self.location.position,
        ];
        CAMERA.read().draw_outline(&points, color);
    }

}
