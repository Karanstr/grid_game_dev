use std::f32::consts::PI;
use macroquad::prelude::*;
use crate::graph::{NodePointer, SparseDirectedGraph};
pub use crate::graph::Index;
//Clean up this import stuff
mod physics;
use physics::*;

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
    fn new(node_pointer:NodePointer, cell:UVec2, depth:u32) -> Self {
        Self {
            node_pointer,
            cell,
            depth
        }
    }
}

pub struct Object {
    root : NodePointer,
    position : Vec2,
    grid_length : f32,
    velocity : Vec2,
    rotation : f32, // >:( iykyk
    angular_velocity : f32,

}

impl Object {

    pub fn new(root:NodePointer, position:Vec2, grid_length:f32) -> Self {
        Self {
            root,
            position,
            grid_length,
            velocity : Vec2::ZERO,
            rotation : 0.0,
            angular_velocity : 0.,
        }
    }

    fn cell_length(&self, depth:u32) -> f32 {
        self.grid_length / 2f32.powf(depth as f32)
    }

    fn cell_top_left_corner(&self, zorder:u32, depth:u32) -> Vec2 {
        let cell_length = self.cell_length(depth);
        let grid_cell = Zorder::to_cell(zorder, depth);
        grid_cell.as_vec2() * cell_length + self.position - self.grid_length/2.
    }

    //Change to relative position?
    fn coord_to_cell(&self, point:Vec2, depth:u32) -> [Option<UVec2>; 4] {
        let mut four_points = [None; 4];
        let half_length = self.grid_length/2.;
        let cell_length = self.cell_length(depth);
        let top_left = self.position - half_length;
        let bottom_right = self.position + half_length;
        let offset = 0.01;
        for i in 0 .. 4 {
            let direction = Vec2::new(
                if i & 0b1 == 1 { 1. } else { -1. },
                if i & 0b10 == 0b10 { 1. } else { -1. }
            );
            let cur_point = point - top_left + offset * direction;
            four_points[i] = if cur_point.clamp(top_left, bottom_right) == cur_point {
                Some( (cur_point / cell_length).floor().as_uvec2() )
            } else { None }
        }
        four_points
    }

    fn find_real_node(&self, world:&World, cell:UVec2, max_depth:u32) -> LimPositionData {
        let max_zorder = Zorder::from_cell(cell, max_depth);
        let (cell_pointer, real_depth) = world.graph.read(self.root, &Zorder::path(max_zorder, max_depth));
        let zorder = max_zorder >> 2 * (max_depth - real_depth);
        LimPositionData::new(cell_pointer, Zorder::to_cell(zorder, real_depth), real_depth)
    }

    //Change to relative position?
    fn get_data_at_position(&self, world:&World, position:Vec2, max_depth:u32) -> [Option<LimPositionData>; 4] {
        let max_depth_cells = self.coord_to_cell(position, max_depth);
        let mut data: [Option<LimPositionData>; 4] = [None; 4];
        for i in 0 .. 4 {
            if let Some(grid_cell) = max_depth_cells[i] {
                data[i] = Some(self.find_real_node(world, grid_cell, max_depth))
            }
        }
        data
    }

    pub fn apply_linear_force(&mut self, force:Vec2) {
        self.velocity += force * Vec2::new(self.rotation.cos(), self.rotation.sin());
    }

    pub fn apply_rotational_force(&mut self, torque:f32) {
        self.angular_velocity += torque
    }

    #[allow(dead_code)]
    pub fn set_rotation(&mut self, new_rotation:f32) {
        self.rotation = new_rotation;
    }

    pub fn draw_facing(&self) {
        draw_vec_line(self.position, self.position + 10. * Vec2::new(self.rotation.cos(), self.rotation.sin()), 1., YELLOW);
    }

}

use vec_friendly_drawing::*;

pub struct World {
    pub blocks : BlockPallete,
    pub graph : SparseDirectedGraph,
}

impl World {

    pub fn new() -> Self {
        Self {
            blocks : BlockPallete::new(),
            graph : SparseDirectedGraph::new(8),
        }
    }

    pub fn render(&self, object:&Object, draw_lines:bool) {
        let filled_blocks = self.graph.dfs_leaves(object.root);
        for (zorder, depth, index) in filled_blocks {
            match self.index_color(index) {
                Some(color) => {
                    let top_left_corner = object.cell_top_left_corner(zorder, depth);
                    draw_square(top_left_corner, object.cell_length(depth), color);
                    if draw_lines { outline_square(top_left_corner, object.cell_length(depth), 1., WHITE) }
                }
                None => { eprintln!("Failed to draw {}, unregistered block", *index) }
            }
        }
    }

    pub fn move_with_collisions(&mut self, moving:&mut Object, hitting:&Object, max_depth:u32) {
        if moving.velocity.length() != 0. {
            dbg!("COLLISION GO");
            let half_length = moving.grid_length / 2.;
            let mut cur_pos = moving.position;
            let mut cur_vel = moving.velocity;
            let mut all_walls_hit = IVec2::ZERO;
            let mut it_count = 0;
            //Find all collisions
            loop {
                it_count += 1;
                dbg!(it_count);
                let mut corners = Vec::from([
                    (Particle::new(cur_pos + Vec2::new(half_length, half_length), cur_vel,Configurations::BottomRight), None),
                    (Particle::new(cur_pos + Vec2::new(-half_length, half_length), cur_vel, Configurations::BottomLeft), None),
                    (Particle::new(cur_pos + Vec2::new(half_length, -half_length), cur_vel, Configurations::TopRight), None),
                    (Particle::new(cur_pos + Vec2::new(-half_length, -half_length), cur_vel, Configurations::TopLeft), None),
                ]);
                for (corner, pos_data) in corners.iter_mut() {
                    *pos_data = hitting.get_data_at_position(&self, corner.position, max_depth)[Zorder::from_configured_direction(-corner.velocity, corner.configuration)];
                }
                let mut vel_left_when_hit = Vec2::ZERO;
                let mut walls_hit = IVec2::ZERO;
                loop {
                    if corners.len() == 0 { break }
                    let cur_corner_index = {
                        let mut min_vel = vel_left_when_hit;
                        let mut cur_corner_index = 100;
                        let mut found = false;
                        for corner_index in 0 .. corners.len() {
                            if corners[corner_index].0.velocity.length() <= min_vel.length() { continue }
                            found = true;
                            cur_corner_index = corner_index;
                            min_vel = corners[cur_corner_index].0.velocity;
                        }
                        if found { cur_corner_index } else { break }
                    };
                    let (cur_point, cur_pos_data) = &mut corners[cur_corner_index];
                    let hit_point = cur_point.next_intersection(hitting, *cur_pos_data);
                    if hit_point.ticks_to_hit.abs() >= 1. { 
                        corners.swap_remove(cur_corner_index); 
                        continue
                    }
                    let new_full_pos_data = hitting.get_data_at_position(&self, hit_point.position, max_depth);
                    *cur_pos_data = new_full_pos_data[Zorder::from_configured_direction(cur_point.velocity, cur_point.configuration)];
                    cur_point.velocity -= hit_point.position - cur_point.position;
                    cur_point.position = hit_point.position;
                    if let Some(data) = cur_pos_data {
                        match self.index_collision(data.node_pointer.index) {
                            Some(OnTouch::Ignore) => {}
                            Some(OnTouch::Resist(possible_hit_walls)) => {
                                walls_hit = {
                                    let no_checks = possible_hit_walls * hit_point.walls_hit;
                                    if no_checks.x == no_checks.y {
                                        let slide_check = self.slide_check(&cur_point, new_full_pos_data);
                                        if slide_check.x == slide_check.y && slide_check.x == 1 {
                                            self.hang_check(moving, 0, 0, hitting, Zorder::from_cell(data.cell, data.depth), data.depth)
                                        } else { slide_check }
                                    } else { no_checks }
                                };
                                if walls_hit == IVec2::ZERO { continue }
                                vel_left_when_hit = cur_point.velocity;
                                corners.swap_remove(cur_corner_index);
                            }
                            None => { eprintln!("Attempting to collide with {}, an unregistered block!", *data.node_pointer.index); }
                        }
                    }
                }
                cur_pos += cur_vel - vel_left_when_hit;
                cur_vel = vel_left_when_hit;
                if walls_hit.x == 1 {
                    cur_vel.x = 0.;
                    all_walls_hit.x = 1;
                }
                if walls_hit.y == 1 {
                    cur_vel.y = 0.;
                    all_walls_hit.y = 1;
                }
                if cur_vel.length() == 0. { break }
            }

            moving.position = cur_pos;
            if all_walls_hit.x == 1 {
                moving.velocity.x = 0.;
            }
            if all_walls_hit.y == 1 {
                moving.velocity.y = 0.;
            }
            let drag = 0.99;
            moving.velocity *= drag;
            let speed_min = 0.005;
            if moving.velocity.x.abs() < speed_min { moving.velocity.x = 0. }
            if moving.velocity.y.abs() < speed_min { moving.velocity.y = 0. }
        }
        moving.rotation += moving.angular_velocity;
        moving.rotation %= 2.*PI;
        moving.angular_velocity = 0.;
    }

    pub fn slide_check(&self, particle:&Particle, position_data:[Option<LimPositionData>; 4]) -> IVec2 {
        //Formalize this with some zorder arithmatic?
        let (x_slide_check, y_slide_check) = if particle.velocity.x < 0. && particle.velocity.y < 0. { //(-,-)
            (2, 1)
        } else if particle.velocity.x < 0. && particle.velocity.y > 0. { //(-,+)
            (0, 3)
        } else if particle.velocity.x > 0. && particle.velocity.y < 0. { //(+,-)
            (3, 0)
        } else { //(+,+)
            (1, 2)
        };
        let x_block_collision = if let Some(pos_data) = position_data[x_slide_check] {
            if let Some(collision) = self.index_collision(pos_data.node_pointer.index) {
                collision
            } else { OnTouch::Ignore }
        } else { OnTouch::Ignore };
        let y_block_collision = if let Some(pos_data) = position_data[y_slide_check] {
            if let Some(collision) = self.index_collision(pos_data.node_pointer.index) {
                collision
            } else { OnTouch::Ignore }
        } else { OnTouch::Ignore };
        IVec2::new(
            if let OnTouch::Resist(_) = y_block_collision { 0 } else { 1 },
            if let OnTouch::Resist(_) = x_block_collision { 0 } else { 1 },
        )
    }

    //Move zorder and depth into single struct
    pub fn hang_check(&self, object1:&Object, corner_zorder:u32, corner_depth:u32, object2:&Object, hitting_zorder:u32, hitting_depth:u32) -> IVec2 {
        let center1 = object1.cell_top_left_corner(corner_zorder, corner_depth) + object1.cell_length(corner_depth)/2.;
        let center2 = object2.cell_top_left_corner(hitting_zorder, hitting_depth) + object2.cell_length(hitting_depth)/2.;
        draw_centered_square(center1, 10., RED);
        draw_centered_square(center2, 10., RED);
        let offset = (center1 - center2).abs();
        if offset.x < offset.y {
            IVec2::new(0, 1)
        } else if offset.x > offset.y {
            IVec2::new(1, 0)
        } else { IVec2::ONE}
    }

    pub fn set_cell_with_mouse(&mut self, modified:&mut Object, mouse_pos:Vec2, depth:u32, index:Index) {
        let block_size = modified.cell_length(depth);
        let rel_mouse_pos = mouse_pos - modified.position;
        let unrounded_cell = rel_mouse_pos / block_size;
        let mut edit_cell = IVec2::new(
            round_away_0_pref_pos(unrounded_cell.x),
            round_away_0_pref_pos(unrounded_cell.y)
        );
        //Clean all this up and find a way to generalize it. Edgecases are the mean.
        if depth != 0 {
            let blocks_on_half = 2i32.pow(depth - 1);
            if edit_cell.abs().max_element() > blocks_on_half { return }
            edit_cell += blocks_on_half;
            if edit_cell.x > blocks_on_half { edit_cell.x -= 1 }
            if edit_cell.y > blocks_on_half { edit_cell.y -= 1 }    
        } else {
            edit_cell = IVec2::ZERO
        }

        let path = Zorder::path( Zorder::from_cell(edit_cell.as_uvec2(), depth), depth );

        if let Ok(root) = self.graph.set_node(modified.root, &path, NodePointer::new(index)) {
            modified.root = root
        } else { error!("Failed to modify cell. Likely means structure is corrupted.") };
    }

    fn index_collision(&self, index:Index) -> Option<OnTouch> {
        if self.blocks.blocks.len() > *index {
            Some(self.blocks.blocks[*index].collision)
        } else { None }
    }

    fn index_color(&self, index:Index) -> Option<Color> {
        if self.blocks.blocks.len() > *index {
            Some(self.blocks.blocks[*index].color)
        } else { None }
    }

}


pub struct Zorder;
impl Zorder {

    pub fn to_cell(zorder:u32, depth:u32) -> UVec2 {
        let mut cell = UVec2::ZERO;
        for layer in 0 .. depth {
            cell.x |= (zorder >> (2 * layer) & 0b1) << layer;
            cell.y |= (zorder >> (2 * layer + 1) & 0b1) << layer;
        }
        cell
    }

    pub fn from_cell(cell:UVec2, depth:u32) -> u32 {
        let mut zorder = 0;
        for layer in (0 .. depth).rev() {
            let step = (((cell.y >> layer) & 0b1) << 1 ) | ((cell.x >> layer) & 0b1);
            zorder = (zorder << 2) | step;
        }
        zorder
    }

    pub fn from_configured_direction(direction:Vec2, configuration:Configurations) -> usize {
        let clamped: Vec2 = direction.signum().clamp(Vec2::ZERO, Vec2::ONE);
        if direction.x == 0. {
            2 * clamped.y as usize | if configuration == Configurations::TopLeft || configuration == Configurations::BottomLeft { 1 } else { 0 }
        } else if direction.y == 0. {
            clamped.x as usize | if configuration == Configurations::TopLeft || configuration == Configurations::TopRight { 2 } else { 0 }
        } else {
            2 * clamped.y as usize | clamped.x as usize
        }
    }

    pub fn read(zorder:u32, layer:u32, depth:u32) -> u32 {
        zorder >> (2 * (depth - layer)) & 0b11
    }

    #[allow(dead_code)]
    pub fn divergence_depth(zorder_a:u32, zorder_b:u32, depth:u32) -> Option<u32> {
        for layer in 1 ..= depth {
            if Self::read(zorder_a, layer, depth) != Self::read(zorder_b, layer, depth) {
                return Some(layer)
            }
        }
        None    
    }

    pub fn path(zorder:u32, depth:u32) -> Vec<u32> {
        let mut steps:Vec<u32> = Vec::with_capacity(depth as usize);
        for layer in 1 ..= depth {
            steps.push(Self::read(zorder, layer, depth));
        }
        steps
    }

}

fn round_away_0_pref_pos(number:f32) -> i32 {
    if number < 0. {
        number.floor() as i32
    } else if number > 0. {
        number.ceil() as i32
    }
    else {
        //We don't want to return 0 when we're trying to avoid 0
        //And the name of the function is prefer_positive, so..
        1 
    }
}

#[allow(dead_code)]
mod vec_friendly_drawing {
    use macroquad::prelude::*;

    pub fn draw_square(top_left_corner:Vec2, length:f32, color:Color) {
        draw_rectangle(top_left_corner.x, top_left_corner.y, length, length, color);
    }

    pub fn draw_centered_square(position:Vec2, length:f32, color:Color) {
        let real_pos = position - length/2.;
        draw_rectangle(real_pos.x, real_pos.y, length, length, color);
    }

    pub fn outline_square(position:Vec2, length:f32, line_width:f32, color:Color) {
        draw_rectangle_lines(position.x, position.y, length, length, line_width, color);
    }

    pub fn draw_vec_line(point1:Vec2, point2:Vec2, line_width:f32, color:Color) {
        draw_line(point1.x, point1.y, point2.x, point2.y, line_width, color);
    }

}



