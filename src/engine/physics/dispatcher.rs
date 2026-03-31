use rapier2d::{
    math::Vector,
    parry::query::{
        Contact, ContactManifold, ContactManifoldsWorkspace, PersistentQueryDispatcher, QueryDispatcher, Unsupported
    },
    prelude::{ Pose, Shape }
};
use super::Voxels;
use super::voxel::voxel_manifolds::contact_manifold_voxel_voxel;

pub struct VoxelDispatcher;
impl<ManifoldData, ContactData> PersistentQueryDispatcher<ManifoldData, ContactData>
    for VoxelDispatcher where
    ManifoldData: Default + Clone,
    ContactData: Default + Copy
{

    fn contact_manifolds(
        &self,
        pos12: &Pose,
        shape1: &dyn Shape,
        shape2: &dyn Shape,
        prediction: f32,
        manifolds: &mut Vec<ContactManifold<ManifoldData, ContactData>>,
        _w: &mut Option<ContactManifoldsWorkspace>
    ) -> Result<(), Unsupported> {

        if let Some(shape1) = shape1.downcast_ref::<Voxels>() &&
            let Some(shape2) = shape2.downcast_ref::<Voxels>()
        {
            contact_manifold_voxel_voxel(pos12, shape1, shape2, prediction, manifolds);
        } else { return Err(Unsupported) }

        Ok(())
    }



    // ~~~ USELESS ~~~
    fn contact_manifold_convex_convex(&self, _a: &Pose, _b: &dyn Shape, _c: &dyn Shape, _d: Option<&dyn rapier2d::parry::query::details::NormalConstraints>, _e: Option<&dyn rapier2d::parry::query::details::NormalConstraints>, _f: f32, _g: &mut ContactManifold<ManifoldData, ContactData>) -> Result<(), Unsupported> { Err(Unsupported) }
}

// ~~~ USELESS ~~~
impl QueryDispatcher for VoxelDispatcher {

    /// Contact should return points in local space of both objects
    fn contact(&self, _a: &Pose, _b: &dyn Shape, _c: &dyn Shape, _d: f32) -> Result<Option<Contact>, Unsupported> {
        unimplemented!()
    }

    fn intersection_test( &self, _a: &Pose, _b: &dyn Shape, _c: &dyn Shape) -> Result<bool, Unsupported> {
        unimplemented!()
    }

    fn distance(&self, _a: &Pose, _b: &dyn Shape, _c: &dyn Shape) -> Result<f32, Unsupported> {
        unimplemented!()
    }

    fn closest_points( &self, _a: &Pose, _b: &dyn Shape, _c: &dyn Shape, _d: f32) -> Result<rapier2d::parry::query::ClosestPoints, Unsupported> {
        unimplemented!()
    }

    fn cast_shapes(&self, _a: &Pose, _b: Vector, _c: &dyn Shape, _d: &dyn Shape, _e: rapier2d::parry::query::ShapeCastOptions) -> Result<Option<rapier2d::parry::query::ShapeCastHit>, Unsupported> {
        unimplemented!()
    }

    fn cast_shapes_nonlinear(&self, _a: &rapier2d::parry::query::NonlinearRigidMotion, _b: &dyn Shape, _c: &rapier2d::parry::query::NonlinearRigidMotion, _d: &dyn Shape, _e: f32, _f: f32, _g: bool) -> Result<Option<rapier2d::parry::query::ShapeCastHit>, Unsupported> {
        unimplemented!()
    }

}

