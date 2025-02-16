use super::*;
use serde::{Serialize, Deserialize};
mod render;
mod movement;
// mod serialization;

#[derive(new)]
pub struct EntityPool {
    #[new(value = "Vec::new()")]
    pub entities: Vec<Entity>,
    #[new(value = "0")]
    last_used_id: u32,
}
impl EntityPool {
    /// This is a little iffy bc it modifies last_used_id but doesn't guarantee the entity will be pushed to the pool
    pub fn build_entity(&mut self, location:Location) -> EntityBuilder {
        EntityBuilder::new(self.new_id(), location)
    }
    /// Only use if `entity` came from [EntityPool::build_entity]
    pub fn add_to_pool(&mut self, entity:Entity) {
        self.entities.push(entity);
    }
    fn new_id(&mut self) -> ID {
        self.last_used_id += 1;
        self.last_used_id
    }
    pub fn get_mut_entity(&mut self, id:ID) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|entity| entity.id == id)
    }
    pub fn get_entity(&self, id:ID) -> Option<&Entity> {
        self.entities.iter().find(|entity| entity.id == id)
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
    pub fn new(id:ID, location:Location, orientation:f32) -> Self {
        Self {
            id,
            location,
            rotation: orientation,
            forward: Vec2::from_angle(orientation),
            velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            corners: tree_corners(location.pointer, location.min_cell_length),
        }
    }
}

pub struct EntityBuilder {
    id: ID,
    location: Location,
    rotation: Option<f32>,
    forward: Option<Vec2>,
    velocity: Option<Vec2>,
    angular_velocity: Option<f32>,
}
impl EntityBuilder {
    pub fn new(id: ID, location: Location) -> Self {
        Self {
            id,
            location,
            rotation: None,
            forward: None,
            velocity: None,
            angular_velocity: None,
        }
    }

    pub fn rotation(mut self, rotation: f32) -> Self {
        self.rotation = Some(rotation);
        self.forward = Some(Vec2::from_angle(rotation));
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
    // Replace with an add_to_pool call
    pub fn build(self) -> Entity {
        Entity {
            id: self.id,
            location: self.location,
            rotation: self.rotation.unwrap_or(0.0),
            forward: self.forward.unwrap_or(Vec2::new(1., 0.)),
            velocity: self.velocity.unwrap_or(Vec2::ZERO),
            angular_velocity: self.angular_velocity.unwrap_or(0.0),
            corners: tree_corners(self.location.pointer, self.location.min_cell_length),
        }
    }
}
