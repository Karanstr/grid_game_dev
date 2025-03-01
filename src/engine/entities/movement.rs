use std::f32::consts::PI;
use macroquad::math::Vec2;
use super::{Entity, ExternalPointer};

#[allow(dead_code)]
impl Entity {
    pub fn rel_rotate(&mut self, angle: f32) {
        self.set_rotation(self.rotation + angle);
    }
    pub fn set_rotation(&mut self, angle: f32) { 
        self.rotation = angle.rem_euclid(PI * 2.);
        self.forward = Vec2::from_angle(self.rotation);
        self.recaclulate_corners();
    }
    pub fn apply_forward_velocity(&mut self, speed:f32) { self.velocity += self.forward * speed }
    pub fn apply_perp_velocity(&mut self, speed:f32) { self.velocity += self.forward.perp() * speed }
    pub fn apply_abs_velocity(&mut self, delta:Vec2) { self.velocity += delta; }
    pub fn stop(&mut self) { 
        self.velocity = Vec2::ZERO; 
        self.angular_velocity = 0.0;
    }
    pub fn set_root(&mut self, new_root:ExternalPointer) { 
        self.location.pointer = new_root;
        self.recaclulate_corners();
    }
}