use super::*;
use crate::globals::*;

impl EntityPool {
    pub fn draw_all(&self, outline:bool) {
        for entity in self.entities.iter() {
            let location = &entity.location;
            if CAMERA.read().aabb.intersects(location.to_aabb()) == BVec2::TRUE {
                entity.draw(outline);
            }
        }
    }

}

impl Entity {
    pub fn draw(&self, outline:bool) {
        let rotation = self.forward;
        let point_offset = center_to_edge(self.location.pointer.height, self.location.min_cell_length);
        let points_list: Vec<([Vec2; 4], usize)> = self.corners.iter().filter_map(|cell| {
            if *cell.index == 0 && !outline { None } else { Some(([
                    (cell.points[0] - point_offset).rotate(rotation) + self.location.position,
                    (cell.points[1] - point_offset).rotate(rotation) + self.location.position,
                    (cell.points[2] - point_offset).rotate(rotation) + self.location.position,
                    (cell.points[3] - point_offset).rotate(rotation) + self.location.position
                ], *cell.index
            ))}
        }).collect();
        for (points, index) in points_list {
            CAMERA.read().draw_rectangle_from_corners(&points, BLOCKS.color(index), outline);
        }
    }
}
