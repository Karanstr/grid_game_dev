mod dispatcher;
mod voxel_shape;

use rapier2d::prelude::*;
pub use voxel_shape::Voxels;

pub struct Physics {
    pub rigid_bodies: RigidBodySet,
    pub colliders: ColliderSet,
    
    pipeline: PhysicsPipeline,
    islands: IslandManager,
    broad_phase: BroadPhaseBvh,
    narrow_phase: NarrowPhase,
}
impl Default for Physics {
    fn default() -> Self {
        Self {
            rigid_bodies: RigidBodySet::default(),
            colliders: ColliderSet::default(),

            pipeline: PhysicsPipeline::default(),
            islands: IslandManager::default(),
            broad_phase: BroadPhaseBvh::default(),
            narrow_phase: NarrowPhase::with_query_dispatcher(dispatcher::VoxelDispatcher),
        }
    }
}

impl Physics {

    pub fn tick(&mut self) {
        self.pipeline.step(
            Vector::new(0., 100.),
            &IntegrationParameters::default(),
            &mut self.islands,
            &mut self.broad_phase,
            &mut self.narrow_phase,
            &mut self.rigid_bodies,
            &mut self.colliders,
            &mut ImpulseJointSet::default(),
            &mut MultibodyJointSet::default(),
            &mut CCDSolver::default(),
            &(),
            &(),
        )
    }

}
