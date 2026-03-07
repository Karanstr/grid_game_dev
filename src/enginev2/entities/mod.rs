use crate::{engine::camera::Camera, enginev2::physics::Physics};

pub struct EntityPool;
impl EntityPool {
    pub fn draw_all(&self, physics: &Physics, camera: &Camera) {
        // for entity in self.entities.iter() {
            // entity.draw(phsyics, camera);
        // }
    }
}

pub struct Entity;
impl Entity {
    pub fn draw(&self, physics: &Physics, camera: &Camera) {
        // let point_offset = center_to_edge(self.location.pointer.height, self.location.min_cell_length);
        // let rb = phsyics.get_rb;
        // let rotation = rb.rotation;
        // let position = rb.position;
        // let points_list: Vec<([Vec2; 4], usize)> = self.corners.iter().map(|cell| {
            // ([
             // (cell.points[0] - point_offset).rotate(rotation) + position,
             // (cell.points[1] - point_offset).rotate(rotation) + position,
             // (cell.points[2] - point_offset).rotate(rotation) + position,
             // (cell.points[3] - point_offset).rotate(rotation) + position
            // ], cell.index as usize
            // )
        // }).collect();
        // for (points, index) in points_list {
        //     camera.draw_rectangle_from_corners(
        //         &points,
        //         RED, // BLOCKS.color(index),
        //         true
        //     );
        // }
    }
}
