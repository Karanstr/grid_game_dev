use rapier2d::{
    math::Vector,
    parry::query::{
        ClosestPoints, Contact, ContactManifold, ContactManifoldsWorkspace, PersistentQueryDispatcher, QueryDispatcher, Unsupported
    },
    prelude::{
        Pose, Shape, TypedShape
    }
};
use super::voxels::Voxels;

pub fn downcast<T: Shape>(shape: &dyn Shape) -> Option<&T> {
    if let TypedShape::Custom(shape1) = shape.as_typed_shape() {
        shape1.downcast_ref()
    } else { None }
}

pub struct VoxelDispatcher;
impl<ManifoldData, ContactData> PersistentQueryDispatcher<ManifoldData, ContactData>
    for VoxelDispatcher
where
    ManifoldData: Default + Clone,
    ContactData: Default + Copy,
{
    fn contact_manifolds(
        &self,
        pos12: &Pose,
        shape1: &dyn Shape,
        shape2: &dyn Shape,
        prediction: f32,
        manifolds: &mut Vec<ContactManifold<ManifoldData, ContactData>>,
        _w: &mut Option<ContactManifoldsWorkspace>,
    ) -> Result<(), Unsupported> {
        if let Some(shape1) = downcast::<Voxels>(shape1) &&
           let Some(shape2) = downcast::<Voxels>(shape2)
        {
            contact_manifold_voxel_voxel(&self, pos12, shape1, shape2, prediction, manifolds);

        } else {
            return Err(Unsupported)
        }
        Ok(())
    }

    // ~~~ USELESS ~~~
    fn contact_manifold_convex_convex(&self, _a: &Pose, _b: &dyn Shape, _c: &dyn Shape, _d: Option<&dyn rapier2d::parry::query::details::NormalConstraints>, _e: Option<&dyn rapier2d::parry::query::details::NormalConstraints>, _f: f32, _g: &mut ContactManifold<ManifoldData, ContactData>) -> Result<(), Unsupported> { Err(Unsupported) }

}

fn contact_manifold_voxel_voxel<ManifoldData, ContactData>(
    dispatcher: &VoxelDispatcher,
    pos12: &Pose,
    shape1: &Voxels,
    shape2: &Voxels,
    prediction: f32,
    manifolds: &mut Vec<ContactManifold<ManifoldData, ContactData>>
) {

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

    fn closest_points( &self, _a: &Pose, _b: &dyn Shape, _c: &dyn Shape, _d: f32) -> Result<ClosestPoints, Unsupported> {
        unimplemented!()
    }

    fn cast_shapes(&self, _a: &Pose, _b: Vector, _c: &dyn Shape, _d: &dyn Shape, _e: rapier2d::parry::query::ShapeCastOptions) -> Result<Option<rapier2d::parry::query::ShapeCastHit>, Unsupported> {
        unimplemented!()
    }

    fn cast_shapes_nonlinear(&self, _a: &rapier2d::parry::query::NonlinearRigidMotion, _b: &dyn Shape, _c: &rapier2d::parry::query::NonlinearRigidMotion, _d: &dyn Shape, _e: f32, _f: f32, _g: bool) -> Result<Option<rapier2d::parry::query::ShapeCastHit>, Unsupported> {
        unimplemented!()
    }

}


    // fn contact_manifolds(
    //     &self,
    //     pos12: &Pose,
    //     shape1: &dyn Shape,
    //     shape2: &dyn Shape,
    //     prediction: f32,
    //     manifolds: &mut Vec<ContactManifold<ManifoldData, ContactData>>,
    //     _w: &mut Option<ContactManifoldsWorkspace>,
    // ) -> Result<(), Unsupported> {
    //
    //     manifolds.clear();
    //     let Some(circle1) = downcast::<MyCircle>(shape1) else { return Err(Unsupported) };
    //     let Some(circle2) = downcast::<MyCircle>(shape2) else { return Err(Unsupported) };
    //
    //     let contact = {
    //         let dist = pos12.translation.length() - circle2.0 - circle1.0;
    //
    //         let normal1 = if dist == 0. { Vector::X } else {
    //             pos12.translation.normalize()
    //         };
    //         let normal2 = -(pos12.rotation.inverse() * normal1);
    //
    //         let point1 = normal1 * circle1.0;
    //         let point2 = normal2 * circle2.0;
    //
    //         if dist < prediction {
    //             Some( Contact::new(point1, point2, normal1, normal2, dist) )
    //         } else { None }
    //     };
    //
    //     if let Some(contact) = contact {
    //         let mut manifold = ContactManifold::new();
    //         manifold.local_n1 = contact.normal1;
    //         manifold.local_n2 = contact.normal2;
    //         manifold.points.push( TrackedContact::new(
    //                 contact.point1,
    //                 contact.point2,
    //                 PackedFeatureId::face(0),
    //                 PackedFeatureId::face(0),
    //                 contact.dist
    //         ));
    //         manifolds.push(manifold);
    //     }
    //
    //     Ok(())   
    // }
