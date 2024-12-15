use std::cmp::Ordering;
use super::*;

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

pub struct BlockPallete {
    pub blocks : Vec<Block>
}

impl BlockPallete {
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
                    name : "Dirt".to_owned(),
                    index : Index(2),
                    collision : OnTouch::Resist(BVec2::TRUE),
                    color : BROWN
                },
                Block {
                    name : "Water".to_owned(),
                    index : Index(3),
                    collision : OnTouch::Ignore,
                    color : BLUE
                },
                Block {
                    name : "Metal".to_owned(),
                    index : Index(4),
                    collision : OnTouch::Resist(BVec2::TRUE),
                    color : GRAY
                }
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
    pub rem_displacement : Vec2,
    pub position_data : Option<LimPositionData>,
    pub configuration : Configurations,
}


// Add these trait implementations
impl PartialOrd for Particle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.rem_displacement.length_squared().partial_cmp(&other.rem_displacement.length_squared())
    }
}

impl Ord for Particle {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl PartialEq for Particle {
    fn eq(&self, other: &Self) -> bool {
        self.rem_displacement.length_squared() == other.rem_displacement.length_squared()
    }
}

impl Eq for Particle {} 

impl Particle {

    pub fn new(position:Vec2, rem_displacement:Vec2, configuration:Configurations) -> Self {
        Self {
            position,
            rem_displacement,
            position_data : None,
            configuration
        }
    }

    pub fn hittable_walls(&self) -> BVec2 {
        match self.configuration {
            Configurations::TopLeft => {
                BVec2::new(
                    if self.rem_displacement.x < 0. { true } else { false },
                    if self.rem_displacement.y < 0. { true } else { false }
                )
            }
            Configurations::TopRight => {
                BVec2::new(
                    if self.rem_displacement.x > 0. { true } else { false },
                    if self.rem_displacement.y < 0. { true } else { false }
                )
            }
            Configurations::BottomLeft => {
                BVec2::new(
                    if self.rem_displacement.x < 0. { true } else { false },
                    if self.rem_displacement.y > 0. { true } else { false }
                )
            }
            Configurations::BottomRight => {
                BVec2::new(
                    if self.rem_displacement.x > 0. { true } else { false },
                    if self.rem_displacement.y > 0. { true } else { false }
                )
            }
        }
    }

    pub fn mag_slide_check(&self) -> BVec2 {
        let abs_vel = self.rem_displacement.abs();
        if abs_vel.y < abs_vel.x { 
            BVec2::new(false, true)
        } else if abs_vel.x < abs_vel.y {
            BVec2::new(true, false)
        } else {
            BVec2::TRUE
        }
    }

    pub fn move_to(&mut self, new_position:Vec2, full_pos_data:[Option<LimPositionData>; 4]) {
        self.rem_displacement -= new_position - self.position;
        self.position = new_position;
        self.position_data = full_pos_data[Zorder::from_configured_direction(self.rem_displacement, self.configuration)];
    }

}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Configurations {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight
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
