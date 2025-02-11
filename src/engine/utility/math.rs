use super::*;
pub const FP_EPSILON: f32 = 0.000_01;
const ROTATIONAL_EPSILON: f32 = FP_EPSILON;

pub trait FloatUtils {
    fn approx_eq(self, b:Self) -> bool;
    fn is_zero(self) -> bool;
    fn snap_zero(self) -> Self;
    type ComponentTruth;
    fn greater(self, b:Self) -> Self::ComponentTruth;
    fn greater_eq(self, b:Self) -> Self::ComponentTruth;
    fn less(self, b:Self) -> Self::ComponentTruth;
    fn less_eq(self, b:Self) -> Self::ComponentTruth;
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
    fn less(self, b:Self) -> Self::ComponentTruth { (self - b).snap_zero() < 0. }
    fn less_eq(self, b:Self) -> Self::ComponentTruth { (self - b).snap_zero() <= 0. }
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
    type SignumType = IVec2;
    fn zero_signum(self) -> Self::SignumType { IVec2::new(self.x.zero_signum(), self.y.zero_signum()) }
}

// Decide whether we need a different epsilon for angles
pub trait AngleUtils {
    fn angle_approx_eq(self, b:Self) -> bool;
    fn angle_mod(self, by:Self) -> Self;
    fn normalize_angle(self) -> Self;
}
impl AngleUtils for f32 {
    fn angle_approx_eq(self, b:Self) -> bool { (self - b).abs() < ROTATIONAL_EPSILON }
    fn angle_mod(self, by:Self) -> Self { 
        let r = self % by;
        if r.angle_approx_eq(by) { 0. } else if r.less(0.) { r + by } else { r }
    }
    fn normalize_angle(self) -> Self { self.angle_mod(2. * std::f32::consts::PI) }
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
    // where ω is the angular velocity
    Vec2::new(
        -angular_velocity * offset.y,
        angular_velocity * offset.x
    )
}

