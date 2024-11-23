use macroquad::prelude::*;
//Clean up this import stuff
use crate::game::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OnTouch {
    Ignore,
    Resist(IVec2),
    //...
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum BlockType {
    Air,
    Ground,
    //...
}


#[derive(Debug)]
pub struct Particle {
    pub position : Vec2,
    pub velocity : Vec2,
    configuration : u8,
}

//Eventually replace all these &Object with &Scene, a particle should march through a given scene, hitting any objects in it's path.
impl Particle {

    pub fn new(position:Vec2, velocity:Vec2, configuration:u8) -> Self {
        Self {
            position,
            velocity,
            configuration
        }
    }

    fn next_intersection(&self, object:&Object, cell:UVec2, depth:u32) -> (Vec2, f32, IVec2) {
        let cell_length = object.cell_length(depth);
        let quadrant = (self.velocity.signum() + 0.5).abs().floor();
        let corner = cell.as_vec2() * cell_length + cell_length * quadrant + object.position - object.grid_length/2.;
        let ticks = (corner - self.position) / self.velocity;
        let ticks_to_first_hit = ticks.min_element();
        let walls_hit = if ticks.y < ticks.x {
            IVec2::new(0, 1)
        } else if ticks.x < ticks.y {
            IVec2::new(1, 0)
        } else { IVec2::ONE };
        (self.position + self.velocity * ticks_to_first_hit, ticks_to_first_hit, walls_hit)
    }

    fn slide_check(&self, object:&Object, mut walls_hit:IVec2, position_data:[Option<(BlockType, UVec2, u32)>; 4]) -> IVec2 {
        //Formalize this with some zorder arithmatic.
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
            Some((block, ..)) => object.on_touch(block),
            None => OnTouch::Resist(IVec2::ZERO)
        };
        let y_block_collision = match position_data[y_slide_check] {
            Some((block, ..)) => object.on_touch(block),
            None => OnTouch::Resist(IVec2::ZERO)
        };
        if x_block_collision != y_block_collision {
            if let OnTouch::Resist(_) = x_block_collision { walls_hit.x = 0 }
            if let OnTouch::Resist(_) = y_block_collision { walls_hit.y = 0 }
        }
        walls_hit
    }

    fn next_boundary_in_tick(&mut self, object:&Object, graph:&SparseDirectedGraph, max_depth:u32, first:bool) -> Option<OnTouch> {
        let relevant_cell = Zorder::from_configured_direction(self.velocity, self.configuration);
        let (cur_block_type, mut grid_cell, mut cur_depth) = object.get_data_at_position(graph, self.position, max_depth)[
            if first { Zorder::from_configured_direction(-self.velocity, self.configuration) } else {relevant_cell}
        ]?;
        loop {
            let (new_position, ticks_to_reach, walls_hit) = self.next_intersection(&object, grid_cell, cur_depth);
            dbg!(new_position);
            if ticks_to_reach >= 1. { return None }
            self.velocity -= new_position - self.position;
            self.position = new_position;
            let data = object.get_data_at_position(graph, self.position, max_depth);
            let new_block_type;
            (new_block_type, grid_cell, cur_depth) = data[relevant_cell]?;
            if new_block_type == cur_block_type { continue }
            return match object.on_touch(new_block_type) {
                OnTouch::Ignore => Some(OnTouch::Ignore),
                OnTouch::Resist(_) => {
                    Some(OnTouch::Resist(
                        if walls_hit.x == walls_hit.y {
                            self.slide_check(object, walls_hit, data)
                        } else { walls_hit }
                    ))
                },
            }
        }
    }

    pub fn march_through(&mut self, object:&Object, graph:&SparseDirectedGraph, max_depth:u32) {
        let mut velocity = self.velocity;
        let mut first = true;
        while self.velocity.length() != 0. {
            match self.next_boundary_in_tick(object, graph, max_depth, first) {
                Some(action) => {
                    match action {
                        OnTouch::Ignore => first = false,
                        OnTouch::Resist(walls_hit) => {
                            first = true;
                            if walls_hit.x == 1 {
                                self.velocity.x = 0.;
                                velocity.x = 0.;
                            }
                            if walls_hit.y == 1 {
                                self.velocity.y = 0.;
                                velocity.y = 0.;
                            }
                        }
                    }
                },
                None => break
            }
        }
        self.position += self.velocity;
        self.velocity = velocity;
    }

}

