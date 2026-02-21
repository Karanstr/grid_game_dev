mod render;
mod movement;
// mod serialization;
use serde::{Serialize, Deserialize};
use glam::Vec2;
// use crate::engine::math::Aabb;
use crate::engine::grid::*;
use crate::engine::physics::collisions::{Corners, corner_handling};


#[derive(derive_new::new)]
pub struct EntityPool {
    #[new(value = "Vec::new()")]
    pub entities: Vec<Entity>,
}
impl EntityPool {
    pub fn create_entity(&mut self, location: Location, rotation: f32) -> ID {
        self.entities.push(Entity::new(self.entities.len() as u32, location, rotation));
        self.entities.len() as u32 - 1
    }
    pub fn _add_to_pool(&mut self, entity: Entity) {
        self.entities.push(entity);
    }
    pub fn get_mut_entity(&mut self, id:ID) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|entity| entity.id == id)
    }
    pub fn get_entity(&self, id:ID) -> Option<&Entity> {
        self.entities.iter().find(|entity| entity.id == id)
    }
}

#[derive(Debug, Clone, Copy, derive_new::new, Serialize, Deserialize)]
pub struct Location {
    pub position: Vec2,
    pub pointer: ExternalPointer,
    #[new(value = "Vec2::splat(1.0)")]
    pub min_cell_length: Vec2,
}
// impl Location {
//     pub fn to_aabb(&self) -> Aabb {
//         Aabb::new(self.position, center_to_edge(self.pointer.height, self.min_cell_length))
//     }
// }

pub type ID = u32;
// Chunk and store corner locations in u8s?
pub struct Entity {
    pub id : ID,
    pub location: Location,
    pub rotation: f32,
    pub forward: Vec2,
    pub velocity: Vec2,
    pub angular_velocity: f32,
    pub corners : Vec<Corners>,
}
impl Entity {
    pub fn new(id: ID, location: Location, rotation: f32) -> Self {
        Self {
            id,
            location,
            rotation: rotation,
            forward: Vec2::from_angle(rotation),
            velocity: Vec2::ZERO,
            angular_velocity: 0.,
            corners: corner_handling::tree_corners(location.pointer, location.min_cell_length),
        }
    }

    pub fn recaclulate_corners(&mut self) { self.corners = corner_handling::tree_corners(self.location.pointer, self.location.min_cell_length) }

    // pub fn aabb(&self) -> Option<Aabb> {
    //     let (mut top_left, mut bottom_right) = self.get_extreme_points()?;
    //     top_left += -center_to_edge(self.location.pointer.height, self.location.min_cell_length) + self.location.position;
    //     bottom_right += -center_to_edge(self.location.pointer.height, self.location.min_cell_length) + self.location.position;
    //     Some(Aabb::from_bounds(top_left, bottom_right))
    // }
    //
    // pub fn get_extreme_points(&self) -> Option<(Vec2, Vec2)> {
    //     if self.corners.is_empty() { return None; }
    //
    //     let mut top_left = Vec2::NAN;
    //     let mut bottom_right = Vec2::NAN;
    //
    //     self.corners.iter().filter(|corner| corner.index != 0)
    //         .flat_map(|corner| &corner.points)
    //         .for_each(|pos| 
    //     {
    //         // Update top-left (minimum x and y)
    //         top_left.x = top_left.x.min(pos.x);
    //         top_left.y = top_left.y.min(pos.y);
    //         // Update bottom-right (maximum x and y)
    //         bottom_right.x = bottom_right.x.max(pos.x);
    //         bottom_right.y = bottom_right.y.max(pos.y);
    //     });
    //
    //     Some((top_left, bottom_right))
    // }

}

