use macroquad::prelude::*;
use super::*;

//Split up collision types from notifications
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
    pub walls_hit : IVec2
}

#[derive(Debug)]
pub struct Particle {
    pub position : Vec2,
    pub velocity : Vec2,
    pub configuration : Configurations,
}

//Move these into scene and object
//Eventually replace all these &Object with &Scene, a particle should march through a given scene, hitting any objects in it's path.
impl Particle {

    pub fn new(position:Vec2, velocity:Vec2, configuration:Configurations) -> Self {
        Self {
            position,
            velocity,
            configuration
        }
    }

    pub fn next_intersection(&self, object:&Object, pos_data:Option<LimPositionData>) -> HitPoint {
        let corner = match pos_data {
            Some(data) => {
                let cell_length = object.cell_length(data.depth);
                let quadrant = (self.velocity.signum() + 0.5).abs().floor();
                data.cell.as_vec2() * cell_length + cell_length * quadrant + object.position - object.grid_length/2.
            }
            None => {
                object.position + object.grid_length*(-self.velocity).signum()
            }
        };
        let ticks = (corner - self.position) / self.velocity;
        let ticks_to_first_hit = ticks.min_element();
        let walls_hit = if ticks.y < ticks.x {
            IVec2::new(0, 1)
        } else if ticks.x < ticks.y {
            IVec2::new(1, 0)
        } else { IVec2::ONE };
        HitPoint {
            position : self.position + self.velocity * ticks_to_first_hit, 
            ticks_to_hit : ticks_to_first_hit, 
            walls_hit
        }
    }

    //Figure out this naming
    pub fn slide_check(&self, world:&World, position_data:[Option<LimPositionData>; 4]) -> IVec2 {
        //Formalize this with some zorder arithmatic?
        let (x_slide_check, y_slide_check) = if self.velocity.x < 0. && self.velocity.y < 0. { //(-,-)
            (2, 1)
        } else if self.velocity.x < 0. && self.velocity.y > 0. { //(-,+)
            (0, 3)
        } else if self.velocity.x > 0. && self.velocity.y < 0. { //(+,-)
            (3, 0)
        } else { //(+,+)
            (1, 2)
        };
        let x_block_collision = match position_data[x_slide_check] {
            Some(pos_data) => world.blocks.blocks[*pos_data.node_pointer.index].collision,
            None => OnTouch::Resist(IVec2::ONE)
        };
        let y_block_collision = match position_data[y_slide_check] {
            Some(pos_data) => world.blocks.blocks[*pos_data.node_pointer.index].collision,
            None => OnTouch::Resist(IVec2::ONE)
        };
        if x_block_collision != y_block_collision {
            if let OnTouch::Resist(_) = x_block_collision { IVec2::new(0, 1) }
            else { IVec2::new(1, 0) }
        } else { IVec2::ONE }
    }

}

