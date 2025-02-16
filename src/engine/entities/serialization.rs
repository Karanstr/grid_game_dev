
use super::{Entity, EntityPool, Vec2, Location, ID};
use serde::{Serialize, Deserialize};
use crate::globals::GRAPH;

impl EntityPool {
    pub fn save_entity(&self, id:ID) -> String {
        self.get_entity(id).unwrap().save()
    }
    pub fn load_entity(&mut self, data:String) -> ID {
        let storer: EntityStorer = serde_json::from_str(&data).unwrap();
        let pointer = GRAPH.write().load_object_json(storer.graph);
        let location = Location::new(storer.position, pointer);
        let entity = self.build_entity(location)
            .rotation(storer.rotation)
            .velocity(storer.velocity)
            .angular_velocity(storer.angular_velocity)
            .build();
        let id = entity.id;
        self.add_to_pool(entity);
        id
    }
}

impl Entity {
    pub fn save(&self) -> String {
        serde_json::to_string(&EntityStorer {
            position: self.location.position,
            rotation: self.rotation,
            velocity: self.velocity,
            angular_velocity: self.angular_velocity,
            graph: GRAPH.read().save_object_json(self.location.pointer),
        }).unwrap()
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