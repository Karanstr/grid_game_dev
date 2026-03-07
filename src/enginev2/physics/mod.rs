use rapier2d::prelude::*;
mod query;
mod voxels;

// Physics before entities
pub struct Physics {
    pipeline: PhysicsPipeline,
    islands: IslandManager,
    broadphase: BroadPhaseBvh,
    narrowphase: NarrowPhase,
    rigid_bodies: RigidBodySet,
    colliders: ColliderSet,
}
impl Physics {
    pub fn new() -> Self {
        Self {
            pipeline: PhysicsPipeline::default(),
            islands: IslandManager::default(),
            broadphase: BroadPhaseBvh::default(),
            narrowphase: NarrowPhase::with_query_dispatcher(query::VoxelDispatcher),
            rigid_bodies: RigidBodySet::default(),
            colliders: ColliderSet::default(),
        }
    }

    pub fn tick(&mut self) {
        self.pipeline.step(
            Vector::new(0., 100.),
            &IntegrationParameters::default(),
            &mut self.islands,
            &mut self.broadphase,
            &mut self.narrowphase,
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
