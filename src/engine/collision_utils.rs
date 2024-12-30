use std::cmp::Ordering;
use macroquad::prelude::*;
use crate::engine::graph::NodePointer;
pub use crate::engine::graph::Index;


#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OnTouch {
    Ignore,
    Resist(BVec2),
    //...
}


#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub name : String,
    pub index : Index,
    pub collision : OnTouch,
    pub color : Color
}


pub struct BlockPalette {
    pub blocks : Vec<Block>
}

impl BlockPalette {
    pub fn new() -> Self {
        Self {
            blocks : vec![
                Block {
                    name : "Air".to_owned(),
                    index : Index(0),
                    collision : OnTouch::Ignore,
                    color : BLACK,
                },
                Block {
                    name : "Grass".to_owned(),
                    index : Index(1),
                    collision : OnTouch::Resist(BVec2::TRUE),
                    color : GREEN
                },
                Block {
                    name : "Metal".to_owned(),
                    index : Index(2),
                    collision : OnTouch::Resist(BVec2::TRUE),
                    color : GRAY
                },
                Block {
                    name : "Water".to_owned(),
                    index : Index(3),
                    collision : OnTouch::Resist(BVec2::TRUE),
                    color : BLUE
                },
            ]
        }
    }
}


#[derive(Debug)]
pub struct HitPoint {
    pub position : Vec2,
    pub ticks_to_hit : f32,
}


#[derive(Debug, Clone)]
pub struct Particle {
    pub position : Vec2,
    pub ticks_into_projection : f32,
    pub position_data : Option<LimPositionData>,
    pub configuration : Configurations,
    pub rel_objects : (usize, usize)
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
impl Particle {

    pub fn new(position:Vec2, configuration:Configurations, owner:usize, hitting:usize) -> Self {
        Self {
            position,
            ticks_into_projection : 0.,
            position_data : None,
            configuration, 
            rel_objects : (owner, hitting)
        }
    }

}


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


#[derive(Clone, Copy, Debug)]
pub struct LimPositionData {
    pub node_pointer : NodePointer,
    pub cell : UVec2,
    pub depth : u32
}
impl LimPositionData {
    pub fn new(node_pointer:NodePointer, cell:UVec2, depth:u32) -> Self {
        Self {
            node_pointer,
            cell,
            depth
        }
    }
}
