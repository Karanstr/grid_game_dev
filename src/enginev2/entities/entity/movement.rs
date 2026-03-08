use glam::Vec2;
use rapier2d::math::Rot2;

use crate::enginev2::physics::Physics;


impl super::Entity {
    pub fn rotate_by(&self, delta: f32, physics: &mut Physics) {
        let collider = physics.colliders.get_mut(self.collider_handle).unwrap();
        collider.set_rotation(Rot2::new(collider.rotation().angle() + delta));
    }

    pub fn move_wrt_rotation(&self, delta: Vec2, physics: &mut Physics) {
        let collider = physics.colliders.get_mut(self.collider_handle).unwrap();
        let rotation = Vec2::from_angle(collider.rotation().angle());
        collider.set_translation(collider.translation() + delta.rotate(rotation));
    }
}
