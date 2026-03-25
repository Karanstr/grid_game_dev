mod entity;
pub use entity::Entity;
use macroquad::color::*;
use crate::engine::{camera::Camera, grid::dim2::*, physics::*};
use lilypads::Pond;

use rapier2d::{math::Pose2, prelude::{ColliderBuilder, SharedShape}};

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
        let collider = ColliderBuilder::new(SharedShape::new(
                Voxels::new(self.graph.clone())
            )).position(position)
        ;
        let collider_handle = physics.colliders.insert(collider);
        let mut entity = Entity::new(None, collider_handle, geometry);
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

    // I don't like this, I'm cheating until I'm sure physics works.
    pub fn run_physics(&self, physics: &Physics, camera: &Camera) {
        let Some(entity1) = self.entities.get(0) else { return };
        let Some(entity2) = self.entities.get(1) else { return };
        let collider1 = physics.colliders.get(entity1.collider_handle).unwrap();
        let collider2 = physics.colliders.get(entity2.collider_handle).unwrap();
        let pos12 = collider1.position().inverse() * collider2.position();
        let shape1 = collider1.shape().downcast_ref::<Voxels>().unwrap();
        let shape2 = collider2.shape().downcast_ref::<Voxels>().unwrap();

        let result = contact_debug_voxel_voxel(&pos12, shape1, shape2);
        for (point, depth, normal) in &result[1] {
            camera.draw_point(*point, 0.1, GOLD);
            camera.draw_vec_line(*point, point + normal * depth * -1., 2., GOLD);
        }
        for (point, depth, normal) in &result[0] {
            camera.draw_point(*point, 0.1, GREEN);
            camera.draw_vec_line(*point, point + normal * depth * -1., 2., GREEN);
        }
    }
}

