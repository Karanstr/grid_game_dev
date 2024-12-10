use super::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OnTouch {
    Ignore,
    Resist(IVec2),
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
                    collision : OnTouch::Resist(IVec2::ONE),
                    color : GREEN
                },
                Block {
                    name : "Dirt".to_owned(),
                    index : Index(2),
                    collision : OnTouch::Resist(IVec2::ONE),
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
                    collision : OnTouch::Resist(IVec2::ONE),
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

#[derive(Debug)]
pub struct Particle {
    pub position : Vec2,
    pub velocity : Vec2,
    pub configuration : Configurations,
}

impl Particle {
    //Add configuration for no configuration, in which all walls are hittable?
    pub fn hittable_walls(&self) -> IVec2 {
        match self.configuration {
            Configurations::TopLeft => {
                IVec2::new(
                    if self.velocity.x < 0. { 1 } else { 0 },
                    if self.velocity.y < 0. { 1 } else { 0 }
                )
            }
            Configurations::TopRight => {
                IVec2::new(
                    if self.velocity.x > 0. { 1 } else { 0 },
                    if self.velocity.y < 0. { 1 } else { 0 }
                )
            }
            Configurations::BottomLeft => {
                IVec2::new(
                    if self.velocity.x < 0. { 1 } else { 0 },
                    if self.velocity.y > 0. { 1 } else { 0 }
                )
            }
            Configurations::BottomRight => {
                IVec2::new(
                    if self.velocity.x > 0. { 1 } else { 0 },
                    if self.velocity.y > 0. { 1 } else { 0 }
                )
            }
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
