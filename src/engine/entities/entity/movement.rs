use glam::Vec2;

use crate::engine::physics::Physics;


impl super::Entity {
    pub fn rotate_by(&self, delta: f32, physics: &mut Physics) {
        let body = physics.rigid_bodies.get_mut(self.rb_handle).unwrap();
        body.apply_torque_impulse(delta, true);
    }

    pub fn move_wrt_rotation(&self, delta: Vec2, physics: &mut Physics) {
        let body = physics.rigid_bodies.get_mut(self.rb_handle).unwrap();
        let rotation = Vec2::from_angle(body.rotation().angle());
        body.apply_impulse(delta.rotate(rotation), true);
    }
}
