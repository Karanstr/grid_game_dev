use super::*;
mod render;
mod movement;
#[derive(new)]
pub struct EntityPool {
    #[new(value = "Vec::new()")]
    pub entities: Vec<Entity>,
    #[new(value = "0")]
    id_counter: u32,
}
impl EntityPool {
    pub fn add_entity(&mut self, location:Location, orientation:f32) -> ID {
        self.id_counter += 1;
        self.entities.push(Entity::new(self.id_counter, location, orientation));
        self.id_counter
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
