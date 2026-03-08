mod entity;
pub use entity::Entity;
use crate::enginev2::{camera::Camera, grid::*, physics::*};
use glam::Vec2;
use lilypads::Pond;

use rapier2d::{prelude::{ColliderBuilder, SharedShape}};

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

    pub fn add(&mut self, geometry: DagPointer, position: Vec2, physics: &mut Physics) -> usize {
        let collider = ColliderBuilder::new(SharedShape::new(
            Voxels::new(geometry, self.graph.clone())
        ))
            .translation(position)
            .rotation(-0.5);
        let collider_handle = physics.colliders.insert(collider);
        let entity = Entity::new(None, collider_handle, geometry);
        self.entities.insert(entity)
    }

    pub fn draw_all(&self, physics: &Physics, camera: &Camera) {
        for (_, entity) in self.entities.iter() {
            let pose = *physics.colliders
                .get(entity.collider_handle).unwrap()
                .position()
            ;
            entity.draw(&self.graph.read(), pose, camera);
        }
    }
}

