use super::*;
//Replace with f32 epsilon constant?
const EPSILON: f32 = 1e-6;

pub trait FloatUtils { 
    fn approx_eq(self, b:Self) -> bool;
    fn is_zero(self) -> bool;
    fn remove_err(self) -> Self;
    type ComponentTruth;
    fn greater(self, b:Self) -> Self::ComponentTruth;
    fn greater_eq(self, b:Self) -> Self::ComponentTruth;
    fn less(self, b:Self) -> Self::ComponentTruth;
    fn less_eq(self, b:Self) -> Self::ComponentTruth;
    type SignumType;
    fn zero_signum(self) -> Self::SignumType;
}
impl FloatUtils for f32 {
    fn approx_eq(self, b:Self) -> bool { (self - b).abs() < EPSILON }
    fn is_zero(self) -> bool { self.approx_eq(0.0) }
    fn remove_err(self) -> Self { if self.is_zero() { 0. } else { self } }
    type ComponentTruth = bool;
    fn greater(self, b:Self) -> Self::ComponentTruth { (self - b).remove_err() > 0. }
    fn greater_eq(self, b:Self) -> Self::ComponentTruth { (self - b).remove_err() >= 0. }
    fn less(self, b:Self) -> Self::ComponentTruth { (self - b).remove_err() < 0. }
    fn less_eq(self, b:Self) -> Self::ComponentTruth { (self - b).remove_err() <= 0. }
    type SignumType = i32;
    fn zero_signum(self) -> Self::SignumType { if self.is_zero() { 0 } else { self.signum() as i32 } }
}
impl FloatUtils for Vec2 {
    fn approx_eq(self, b:Self) -> bool { self.x.approx_eq(b.x) && self.y.approx_eq(b.y) }
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
    fn remove_err(self) -> Self {
        Vec2::new(self.x.remove_err(), self.y.remove_err())     
    }
    type SignumType = IVec2;
    fn zero_signum(self) -> Self::SignumType { 
        IVec2::new(self.x.zero_signum(), self.y.zero_signum())     
    }
    fn is_zero(self) -> bool { self.x.is_zero() && self.y.is_zero() }
}

// Idk if I trust this, look through it properly later.
pub fn mod_with_err(x: f32, m: f32) -> f32 {
    let r = x % m;
    if r.approx_eq(m) { 0. } else if r < 0. { r + m } else { r }
}
