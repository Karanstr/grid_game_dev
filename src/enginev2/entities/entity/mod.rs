use rapier2d::prelude::{ColliderHandle, RigidBodyHandle};

use crate::enginev2::{grid::DagPointer, physics::{Physics, Voxels}};

mod movement;
mod render;

pub struct Entity {
    rb_handle: Option<RigidBodyHandle>,
    pub collider_handle: ColliderHandle,

    geometry: DagPointer,
}
impl Entity {

    pub fn new(rb_handle: Option<RigidBodyHandle>, collider_handle: ColliderHandle, geometry: DagPointer) -> Self {
        Self {
            rb_handle,
            collider_handle,
            geometry,
        }
    }
    
    pub fn set_geometry(&mut self, geometry: DagPointer, physics: &mut Physics) {
        self.geometry = geometry;

        let shape = physics.colliders
            .get_mut(self.collider_handle).unwrap()
            .shape_mut()
            .downcast_mut::<Voxels>().unwrap()
        ;
        shape.geometry = geometry;
    }

}

