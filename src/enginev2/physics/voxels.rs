use std::f32::consts::FRAC_1_SQRT_2;

// Voxels should have minimum size and trees are based on height.


use rapier2d::{
    math::Vector,
    prelude::{
        PointQuery, RayCast, Shape, Aabb
    }
};

// Rectify relationship with ExternalPointer
// Figure out what we actually need to know about this shape
pub struct Voxels {
    pub head: u32,
    pub height: u32,
}
impl Voxels {
    pub fn new(head: u32, height: u32) -> Self {
        Self {
            head, 
            height
        }
    }
}
impl Shape for Voxels {
    
    fn compute_local_aabb(&self) -> Aabb {
        let half_extents = Vector::splat(2u32.pow(self.height) as f32 / 2.);
        Aabb::from_half_extents(Vector::ZERO, half_extents)
    }

    fn compute_local_bounding_sphere(&self) -> rapier2d::parry::bounding_volume::BoundingSphere {
        let side_length = 2u32.pow(self.height) as f32;
        rapier2d::parry::bounding_volume::BoundingSphere::new(Vector::ZERO, side_length / FRAC_1_SQRT_2)
    }

    fn mass_properties(&self, density: f32) -> rapier2d::prelude::MassProperties {
        rapier2d::prelude::MassProperties::new(Vector::ZERO, density, density)
    }
    
    fn shape_type(&self) -> rapier2d::prelude::ShapeType {
        rapier2d::prelude::ShapeType::Custom
    }
    fn as_typed_shape(&self) -> rapier2d::prelude::TypedShape<'_> {
        rapier2d::prelude::TypedShape::Custom(self)
    }

    // ~~~ USELESS ~~~
    
    fn clone_dyn(&self) -> Box<dyn Shape> { unimplemented!() }
    fn scale_dyn(&self, _a: Vector, _b: u32) -> Option<Box<dyn Shape>> { None }
    
    // Don't need them until CCD
    fn ccd_thickness(&self) -> f32 { 0. }
    fn ccd_angular_thickness(&self) -> f32 { 0. }

}

impl PointQuery for Voxels { 
    
    fn project_local_point(&self, _: rapier2d::prelude::Vector, _: bool) -> rapier2d::parry::query::PointProjection {
        unimplemented!()
    }

    fn project_local_point_and_get_feature(&self, _: rapier2d::prelude::Vector) -> (rapier2d::parry::query::PointProjection, rapier2d::prelude::FeatureId) {
        unimplemented!()
    }

}

impl RayCast for Voxels {
    fn cast_local_ray_and_get_normal(&self, _: &rapier2d::parry::query::Ray, _: f32, _: bool) -> Option<rapier2d::parry::query::RayIntersection> {
        unimplemented!()
    }
}
