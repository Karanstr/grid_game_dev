use std::f32::consts::FRAC_1_SQRT_2;
use rapier2d::{
    math::Vector,
    prelude::{Aabb, PointQuery, RayCast, Shape}
};

impl Shape for super::Voxels {
    
    fn compute_local_aabb(&self) -> Aabb {
        Aabb::new(Vector::ZERO, Vector::splat(Self::length(self.geometry.height)))
    }

    fn compute_local_bounding_sphere(&self) -> rapier2d::parry::bounding_volume::BoundingSphere {
        let side_length = Self::length(self.geometry.height);
        let center = Vector::splat(side_length / 2.);
        rapier2d::parry::bounding_volume::BoundingSphere::new(center, side_length / FRAC_1_SQRT_2)
    }

    // For now sets center of mass as center of the grid
    fn mass_properties(&self, density: f32) -> rapier2d::prelude::MassProperties {
        let local_com = Vector::splat(Self::length(self.geometry.height) / 2.);
        rapier2d::prelude::MassProperties::new(local_com, density, density)
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

// ~~~ USELESS ~~~
impl PointQuery for super::Voxels { 
    
    fn project_local_point(&self, _: rapier2d::prelude::Vector, _: bool) -> rapier2d::parry::query::PointProjection {
        unimplemented!()
    }

    fn project_local_point_and_get_feature(&self, _: rapier2d::prelude::Vector) -> (rapier2d::parry::query::PointProjection, rapier2d::prelude::FeatureId) {
        unimplemented!()
    }

}

impl RayCast for super::Voxels {
    fn cast_local_ray_and_get_normal(&self, _: &rapier2d::parry::query::Ray, _: f32, _: bool) -> Option<rapier2d::parry::query::RayIntersection> {
        unimplemented!()
    }
}
