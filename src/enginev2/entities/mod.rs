use crate::enginev2::{camera::Camera, physics::Physics};
use lilypads::Pond;

pub struct EntityPool {
    entities: Pond<Entity>,
    graph: crate::GRAPH
}
impl EntityPool {

    pub fn new(graph: crate::GRAPH) -> Self {
        Self {
            entities: Pond::new(),
            graph,
        }
    }

    pub fn draw_all(&self, physics: &Physics, camera: &Camera) {
        for (_, entity) in self.entities.iter() {
            entity.draw(physics, camera);
        }
    }
}

pub struct Entity;
impl Entity {
    pub fn draw(&self, physics: &Physics, camera: &Camera) {
        // let point_offset = center_to_edge(self.location.pointer.height, self.location.min_cell_length);
        // let rb = phsyics.get_rb;
        // let rotation = rb.rotation;
        // let position = rb.position;
        // Should be able to use Pose and basically do this for free?
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
        //     );
        // }
    }
}
