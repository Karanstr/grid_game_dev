mod entity;
pub use entity::Entity;
use crate::engine::{camera::Camera, grid::*, physics::*};
use lilypads::Pond;

use rapier2d::{math::Pose2, prelude::{ColliderBuilder, SharedShape}};

// https://docs.rs/hecs/latest/hecs/

pub struct EntityPool {
    entities: Pond<Entity>,
    pub graph: crate::GRAPH,
}
impl EntityPool {

    pub fn new(graph: crate::GRAPH) -> Self {
        Self {
            entities: Pond::new(),
            graph,
        }
    }

    pub fn get(&self, idx: usize) -> Option<&Entity> {
        self.entities.get(idx)
    }

    pub fn add(&mut self, geometry: DagPointer, position: Pose2, physics: &mut Physics) -> usize {
        let collider = ColliderBuilder::new(SharedShape::new(
                Voxels::new(self.graph.clone())
            )).position(position)
        ;
        let collider_handle = physics.colliders.insert(collider);
        let mut entity = Entity::new(None, collider_handle, geometry);
        entity.set_geometry(geometry, physics);
        self.entities.insert(entity)
    }

    pub fn draw_all(&self, physics: &mut Physics, camera: &Camera) {
        for (_, entity) in self.entities.iter() {
            let collider = physics.colliders.get_mut(entity.collider_handle).unwrap();
            let pose = *collider.position();
            let shape = collider.shape_mut().downcast_mut::<Voxels>().unwrap();
            entity.draw(&self.graph.read(), pose, camera, &shape.faces);
        }
    }
}

