use std::cmp::Ordering;
use derive_new::new;
use macroquad::math::Vec2;
use crate::grid::CellData;


#[derive(Debug, Clone, new)]
pub struct Particle {
    pub position : Vec2,
    #[new(value = "0.")]
    pub ticks_into_projection : f32,
    #[new(value = "None")]
    pub position_data : Option<CellData>,
    pub configuration : Configurations,
    pub owner : usize,
    pub hitting : usize,
}
impl PartialOrd for Particle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ticks_into_projection.partial_cmp(&other.ticks_into_projection)
    }
}
impl Ord for Particle {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl PartialEq for Particle {
    fn eq(&self, other: &Self) -> bool {
        self.ticks_into_projection == other.ticks_into_projection
    }
}
impl Eq for Particle {} 


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Configurations {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight
}
impl Configurations {
    pub fn from_index(index:usize) -> Self {
        match index {
            0 => Self::TopLeft,
            1 => Self::TopRight,
            2 => Self::BottomLeft,
            3 => Self::BottomRight,
            _ => panic!("Invalid Configuration Index")
        }
    }
}
