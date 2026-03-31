mod entity;
pub use entity::Entity;
use crate::engine::{camera::Camera, grid::dim2::*, physics::*};
use lilypads::Pond;

use rapier2d::{math::Pose2, prelude::{ColliderBuilder, RigidBodyBuilder, SharedShape}};

// https://docs.rs/hecs/latest/hecs/ ?

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

    pub fn len(&self) -> usize { self.entities.len() }

    pub fn get(&self, idx: usize) -> Option<&Entity> {
        self.entities.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Entity> {
        self.entities.get_mut(idx)
    }

    pub fn add(&mut self, geometry: DagPointer, position: Pose2, physics: &mut Physics) -> usize {
        let rb_handle = physics.rigid_bodies.insert(
            RigidBodyBuilder::dynamic()
                .pose(position)
                .linear_damping(0.8)
                .angular_damping(0.8)
            .build()
        );

        let collider = ColliderBuilder::new(SharedShape::new(
            Voxels::new(self.graph.clone())
        ));
        let collider_handle = physics.colliders.insert_with_parent(collider, rb_handle, &mut physics.rigid_bodies);
        let mut entity = Entity::new(rb_handle, collider_handle, geometry);
        entity.set_geometry(geometry, physics);
        self.entities.insert(entity)
    }

    pub fn draw_all(&self, physics: &Physics, camera: &Camera) {
        for (_, entity) in self.entities.iter() {
            let pose = *physics.colliders.get(entity.collider_handle).unwrap().position();
            let leaves = dfs_leaves(self.graph.read().nodes.unsafe_data(), entity.geometry.head);
            entity.draw(pose, camera, &leaves, 0.5);
        }
    }

}
