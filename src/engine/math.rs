use super::*;
// pub const FP_EPSILON: f32 = 0.001;
pub const FP_EPSILON: f32 = f32::EPSILON;

#[derive(Debug, Clone, Copy, new)]
pub struct Aabb {
    center: Vec2,
    radius: Vec2
}
impl Aabb {

    pub fn from_bounds(top_left:Vec2, bottom_right:Vec2) -> Self {
        Self {
            center: (top_left + bottom_right) / 2.,
            radius: (bottom_right - top_left) / 2.,
        }
    }

    pub fn min(&self) -> Vec2 { self.center - self.radius }
    pub fn max(&self) -> Vec2 { self.center + self.radius }

    pub fn center(&self) -> Vec2 { self.center }
    pub fn radius(&self) -> Vec2 { self.radius }

    pub fn intersects(&self, other:Self) -> BVec2 {
        (other.center - self.center).less_eq_mag(self.radius + other.radius)
    }
    pub fn contains(&self, point:Vec2) -> BVec2 {
        (point - self.center).less_eq_mag(self.radius)
    }
    
    pub fn move_by(&mut self, displacement:Vec2) { self.center += displacement }
    pub fn move_to(&mut self, position:Vec2) { self.center = position }
    
    pub fn expand(&self, distance:Vec2) -> Self {
        Self {
            center: self.center + distance / 2.,
            radius: self.radius + distance.abs() / 2.,
        }
    }
    pub fn shrink(&self, distance:Vec2) -> Self {
        Self {
            center: self.center - distance / 2.,
            radius: (self.radius - distance.abs() / 2.).abs(),
        }
    }

    pub fn exterior_will_intersect(&self, point:Vec2, velocity:Vec2) -> Option<Vec2> {
        let mut walls_will_hit = Vec2::ZERO;
        let top_left = self.min();
        let bottom_right = self.max();

        if point.x.less_eq(top_left.x) {
            if velocity.x.greater(0.) { walls_will_hit.x = -1. } else { return None }
        } else if point.x.greater_eq(bottom_right.x) {
            if velocity.x.less(0.) { walls_will_hit.x = 1. } else { return None }
        }

        // Check y-axis boundaries
        if point.y.less_eq(top_left.y) {
            if velocity.y.greater(0.) { walls_will_hit.y = -1. } else { return None }
        } else if point.y.greater_eq(bottom_right.y) {
            if velocity.y.less(0.) { walls_will_hit.y = 1. } else { return None }
        }
        
        Some(walls_will_hit)
    }
}

pub trait FloatUtils {
    fn approx_eq(self, b:Self) -> bool;
    fn is_zero(self) -> bool;
    fn snap_zero(self) -> Self;
    type ComponentTruth;
    fn greater(self, b:Self) -> Self::ComponentTruth;
    fn greater_eq(self, b:Self) -> Self::ComponentTruth;
    fn greater_mag(self, b:Self) -> Self::ComponentTruth;
    fn greater_eq_mag(self, b:Self) -> Self::ComponentTruth;
    fn less(self, b:Self) -> Self::ComponentTruth;
    fn less_eq(self, b:Self) -> Self::ComponentTruth;
    fn less_mag(self, b:Self) -> Self::ComponentTruth;
    fn less_eq_mag(self, b:Self) -> Self::ComponentTruth;
    type SignumType;
    fn zero_signum(self) -> Self::SignumType;
}
impl FloatUtils for f32 {
    fn approx_eq(self, b:Self) -> bool { (self - b).abs() < FP_EPSILON }
    fn is_zero(self) -> bool { self.approx_eq(0.0) }
    fn snap_zero(self) -> Self { if self.is_zero() { 0. } else { self } }
    type ComponentTruth = bool;
    fn greater(self, b:Self) -> Self::ComponentTruth { (self - b).snap_zero() > 0. }
    fn greater_eq(self, b:Self) -> Self::ComponentTruth { (self - b).snap_zero() >= 0. }
    fn greater_mag(self, b:Self) -> Self::ComponentTruth { (self.abs() - b.abs()).snap_zero() > 0. }
    fn greater_eq_mag(self, b:Self) -> Self::ComponentTruth { (self.abs() - b.abs()).snap_zero() >= 0. }
    fn less(self, b:Self) -> Self::ComponentTruth { (self - b).snap_zero() < 0. }
    fn less_eq(self, b:Self) -> Self::ComponentTruth { (self - b).snap_zero() <= 0. }
    fn less_mag(self, b:Self) -> Self::ComponentTruth { (self.abs() - b.abs()).snap_zero() < 0. }
    fn less_eq_mag(self, b:Self) -> Self::ComponentTruth { (self.abs() - b.abs()).snap_zero() <= 0. }
    type SignumType = i32;
    fn zero_signum(self) -> Self::SignumType { if self.is_zero() { 0 } else { self.signum() as i32 } }
}
impl FloatUtils for Vec2 {
    fn approx_eq(self, b:Self) -> bool { self.x.approx_eq(b.x) && self.y.approx_eq(b.y) }
    fn is_zero(self) -> bool { self.x.is_zero() && self.y.is_zero() }
    fn snap_zero(self) -> Self { Vec2::new(self.x.snap_zero(), self.y.snap_zero()) }
    type ComponentTruth = BVec2;
    fn greater(self, b:Self) -> Self::ComponentTruth {
        BVec2::new(
            self.x.greater(b.x),
            self.y.greater(b.y)
        )
    }
    fn greater_eq(self, b:Self) -> Self::ComponentTruth {
        BVec2::new(
            self.x.greater_eq(b.x),
            self.y.greater_eq(b.y)
        )
    }
    fn greater_mag(self, b:Self) -> Self::ComponentTruth { 
        BVec2::new(
            self.x.greater_mag(b.x),
            self.y.greater_mag(b.y)
        )
    }
    fn greater_eq_mag(self, b:Self) -> Self::ComponentTruth {   
        BVec2::new(
            self.x.greater_eq_mag(b.x),
            self.y.greater_eq_mag(b.y)
        )
    }
    fn less(self, b:Self) -> Self::ComponentTruth { 
        BVec2::new(
            self.x.less(b.x),
            self.y.less(b.y)
        )
    }
    fn less_eq(self, b:Self) -> Self::ComponentTruth { 
        BVec2::new(
            self.x.less_eq(b.x),
            self.y.less_eq(b.y)
        )
    }
    fn less_mag(self, b:Self) -> Self::ComponentTruth { 
        BVec2::new(
            self.x.less_mag(b.x),
            self.y.less_mag(b.y)
        )
    }
    fn less_eq_mag(self, b:Self) -> Self::ComponentTruth { 
        BVec2::new(
            self.x.less_eq_mag(b.x),
            self.y.less_eq_mag(b.y)
        )
    }
    type SignumType = IVec2;
    fn zero_signum(self) -> Self::SignumType { IVec2::new(self.x.zero_signum(), self.y.zero_signum()) }
}

pub trait BVecUtils {
    fn as_vec2(self) -> Vec2;
}
impl BVecUtils for BVec2 {
    fn as_vec2(self) -> Vec2 { Vec2::new(
        if self.x { 1. } else { 0. }, 
        if self.y { 1. } else { 0. }) 
    }
}

/// Converts angular velocity to tangential velocity for a point offset from the center of rotation.
/// 
/// # Arguments
/// * `angular_velocity` - Angular velocity in radians per second
/// * `offset` - Vector from the center of rotation to the point (x, y components)
/// 
/// # Returns
/// A Vec2 representing the tangential velocity (x, y components)
pub fn angular_to_tangential_velocity(angular_velocity: f32, offset: Vec2) -> Vec2 {
    // For a point at position (x, y) relative to center of rotation,
    // the tangential velocity components are:
    // vx = -ω * y
    // vy = ω * x

    Vec2::new(
        -angular_velocity * offset.y,
        angular_velocity * offset.x
    )
}

