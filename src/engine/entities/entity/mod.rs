use glam::{UVec2, Vec2};
use rapier2d::prelude::{ColliderHandle, RigidBodyHandle};

use crate::engine::{grid::dim2::*, physics::{Physics, Voxels}};

mod movement;
mod render;

pub struct Entity {
    pub rb_handle: RigidBodyHandle,
    pub collider_handle: ColliderHandle,

    pub geometry: DagPointer,
}
impl Entity {

    pub fn new(rb_handle: RigidBodyHandle, collider_handle: ColliderHandle, geometry: DagPointer) -> Self {
        Self {
            rb_handle,
            collider_handle,
            geometry,
        }
    }

    pub fn set_geometry(&mut self, geometry: DagPointer, physics: &mut Physics) {
        self.geometry = geometry;

        physics.colliders
            .get_mut(self.collider_handle).unwrap()
            .shape_mut()
            .downcast_mut::<Voxels>().unwrap()
            .update_shape(geometry)
        ;
    }

    /// This is a bad function because it doesn't verify that the pointer in set is valid.
    /// I don't care right now.
    pub fn set_block(&mut self, physics: &mut Physics, world_point: Vec2, set: DagPointer) -> bool {
        if set.height > self.geometry.height { return false; }
        let collider = physics.colliders.get_mut(self.collider_handle).unwrap();
        let pose = *collider.position();
        let shape = collider.shape_mut().downcast_mut::<Voxels>().unwrap();
        let mut length = Voxels::length(self.geometry.height);
        let mut local_point = pose.inverse() * world_point;
        if local_point.min_element() < 0. || local_point.max_element() >= length { return false; }

        let mut path = Vec::new();
        for _ in set.height .. self.geometry.height {
            length /= 2.;
            let subcell = [(local_point.x > length) as u32, (local_point.y > length) as u32];
            local_point -= length * UVec2::from(subcell).as_vec2();
            path.push(Cell::from_internal(subcell));
        }
        
        let new_head = shape.graph.write().set_node(self.geometry.head, &path, set.head);
        self.set_geometry(DagPointer::new(new_head, self.geometry.height), physics);
        true
    }

}

