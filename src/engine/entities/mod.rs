use super::*;
use serde::{Serialize, Deserialize};
mod render;
mod movement;
mod serialization;

#[derive(new)]
pub struct EntityPool {
    #[new(value = "Vec::new()")]
    pub entities: Vec<Entity>,
}
impl EntityPool {
    pub fn add_to_pool(&mut self, entity:Entity) {
        self.entities.push(entity);
    }
    pub fn get_mut_entity(&mut self, id:ID) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|entity| entity.id == id)
    }
    pub fn get_entity(&self, id:ID) -> Option<&Entity> {
        self.entities.iter().find(|entity| entity.id == id)
    }
}

#[derive(Debug, Clone, Copy, new, Serialize, Deserialize)]
pub struct Location {
    pub position: Vec2,
    pub pointer: ExternalPointer,
    #[new(value = "Vec2::splat(1.0)")]
    pub min_cell_length: Vec2,
}
impl Location {
    pub fn to_aabb(&self) -> Aabb {
        Aabb::new(self.position, center_to_edge(self.pointer.height, self.min_cell_length))
    }
}

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
    pub fn recaclulate_corners(&mut self) { self.corners = tree_corners(self.location.pointer, self.location.min_cell_length) }
    pub fn aabb(&self) -> Aabb {
        let (mut top_left, mut bottom_right) = self.get_extreme_points();
        top_left += -center_to_edge(self.location.pointer.height, self.location.min_cell_length) + self.location.position;
        bottom_right += -center_to_edge(self.location.pointer.height, self.location.min_cell_length) + self.location.position;
        Aabb::from_bounds(top_left, bottom_right)
    }

    pub fn get_extreme_points(&self) -> (Vec2, Vec2) {
        if self.corners.is_empty() {
            return (self.location.position, self.location.position);
        }

        let mut top_left = Vec2::NAN;
        let mut bottom_right = Vec2::NAN;
        
        self.corners.iter().filter(|corner| *corner.index != 0)
            .flat_map(|corner| &corner.points)
            .for_each(|pos| {
                // Update top-left (minimum x and y)
                top_left.x = top_left.x.min(pos.x);
                top_left.y = top_left.y.min(pos.y);
                // Update bottom-right (maximum x and y)
                bottom_right.x = bottom_right.x.max(pos.x);
                bottom_right.y = bottom_right.y.max(pos.y);
            });
            (top_left, bottom_right)
        }

}

pub struct EntityBuilder {
    id: ID,
    location: Location,
    rotation: Option<f32>,
    velocity: Option<Vec2>,
    angular_velocity: Option<f32>,
}
impl EntityBuilder {
    pub fn new(id: ID, location: Location) -> Self {
        Self {
            id,
            location,
            rotation: None,
            velocity: None,
            angular_velocity: None,
        }
    }

    pub fn rotation(mut self, rotation: f32) -> Self {
        self.rotation = Some(rotation);
        self
    }

    pub fn velocity(mut self, velocity: Vec2) -> Self {
        self.velocity = Some(velocity);
        self
    }

    pub fn angular_velocity(mut self, angular_velocity: f32) -> Self {
        self.angular_velocity = Some(angular_velocity);
        self
    }
    
    pub fn build(self) -> Entity {
        Entity {
            id: self.id,
            location: self.location,
            rotation: self.rotation.unwrap_or(0.0),
            forward: Vec2::from_angle(self.rotation.unwrap_or(0.0)),
            velocity: self.velocity.unwrap_or(Vec2::ZERO),
            angular_velocity: self.angular_velocity.unwrap_or(0.0),
            corners: tree_corners(self.location.pointer, self.location.min_cell_length),
        }
    }

}
