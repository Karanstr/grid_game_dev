use rapier2d::math::Pose;
use rapier2d::parry::query::ContactManifold;
use super::super::dispatcher::VoxelDispatcher;
use super::Voxels;

pub fn contact_manifold_voxel_voxel<ManifoldData, ContactData>(
    dispatcher: &VoxelDispatcher,
    pos12: &Pose,
    shape1: &Voxels,
    shape2: &Voxels,
    prediction: f32,
    manifolds: &mut Vec<ContactManifold<ManifoldData, ContactData>>
) {

}
