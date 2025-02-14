use super::*;

impl Entity {
    pub fn rel_rotate(&mut self, angle: f32) {
        self.rotation += angle;
        self.forward = Vec2::from_angle(self.rotation);
        self.recaclulate_corners();
    }
    pub fn set_rotation(&mut self, angle: f32) { 
        self.rotation = angle;
        self.forward = Vec2::from_angle(self.rotation);
        self.recaclulate_corners();
    }
    pub fn apply_forward_velocity(&mut self, speed:f32) { self.velocity += self.forward * speed }
    pub fn apply_perp_velocity(&mut self, speed:f32) { self.velocity += self.forward.perp() * speed }
    pub fn apply_abs_velocity(&mut self, delta:Vec2) { self.velocity += delta; }
    pub fn set_root(&mut self, new_root:ExternalPointer) { 
        self.location.pointer = new_root;
        self.recaclulate_corners();
    }
    pub fn recaclulate_corners(&mut self) { self.corners = tree_corners(self.location.pointer, self.location.min_cell_length) }
}