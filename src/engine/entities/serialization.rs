
use super::{Entity, EntityPool, Vec2, Location, ID, corner_handling};
use serde::{Serialize, Deserialize};
use crate::globals::GRAPH;

impl EntityPool {
    pub fn save_entity(&self, id:ID) -> String {
        self.get_entity(id).unwrap().save()
    }
    
}

impl Entity {
    pub fn save(&self) -> String {
        serde_json::to_string_pretty(&EntityStorer {
            position: self.location.position,
            rotation: self.rotation,
            velocity: self.velocity,
            angular_velocity: self.angular_velocity,
            graph: GRAPH.read().save_object_json(self.location.pointer),
        }).unwrap()
    }
    pub fn load(data:String, id:ID) -> Entity {
        let storer: EntityStorer = serde_json::from_str(&data).unwrap();
        let pointer = GRAPH.write().load_object_json(storer.graph);
        let location = Location::new(storer.position, pointer);
        Entity {
            id,
            location,
            rotation: storer.rotation,
            forward: Vec2::from_angle(storer.rotation),
            velocity: storer.velocity,
            angular_velocity: storer.angular_velocity,
            corners: corner_handling::tree_corners(location.pointer, location.min_cell_length),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct EntityStorer {
    position: Vec2,
    rotation: f32,
    velocity: Vec2,
    angular_velocity: f32,
    graph: String
}