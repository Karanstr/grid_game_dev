use std::f32::consts::PI;
use macroquad::prelude::*;
use crate::graph::{NodePointer, SparseDirectedGraph};
pub use crate::graph::Index;
//Clean up this import stuff
mod physics;
use physics::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Events {
    Wet,
}

pub struct Object {
    name : String,
    root : NodePointer,
    position : Vec2,
    grid_length : f32,
    velocity : Vec2,
    rotation : f32, // >:( iykyk
    angular_velocity : f32,

}

impl Object {

    pub fn new(name:String, root:NodePointer, position:Vec2, grid_length:f32) -> Self {
        Self {
            name,
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

    //Add a struct for this return and slide_check's input?
    fn find_real_node(&self, world:&World, cell:UVec2, max_depth:u32) -> (NodePointer, UVec2, u32) {
        let max_zorder = Zorder::from_cell(cell, max_depth);
        let (cell_pointer, real_depth) = world.graph.read(self.root, &Zorder::path(max_zorder, max_depth));
        let zorder = max_zorder >> 2 * (max_depth - real_depth);
        (cell_pointer, Zorder::to_cell(zorder, real_depth), real_depth)
    }

    //Change to relative position?
    fn get_data_at_position(&self, world:&World, position:Vec2, max_depth:u32) -> [Option<(Index, UVec2, u32)>; 4] {
        let max_depth_cells = self.coord_to_cell(position, max_depth);
        let mut data: [Option<(Index, UVec2, u32)>; 4] = [None; 4];
        for i in 0 .. 4 {
            if let Some(grid_cell) = max_depth_cells[i] {
                let (node, real_cell, depth) = self.find_real_node(world, grid_cell, max_depth);
                data[i] = Some((node.index, real_cell, depth))
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
            let cell_length = object.cell_length(depth);
            let grid_cell = Zorder::to_cell(zorder, depth);
            let offset = grid_cell.as_vec2() * cell_length + object.position - object.grid_length/2.;
            draw_square(offset, cell_length, self.blocks.blocks[*index].color);
            if draw_lines { outline_square(offset, cell_length, 1., WHITE) }
        }
    }

    pub fn move_with_collisions(&mut self, moving:&mut Object, hitting:&Object) {
        if moving.velocity.length() != 0. {
            let mut particle_approximation = Particle::new(moving.position, moving.velocity, 0);
            particle_approximation.march_through(hitting, &self, 5);
            (moving.position, moving.velocity) = (particle_approximation.position, particle_approximation.velocity);
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

    pub fn notify(&self, object:&Object, event:Events) {
        match event {
            Events::Wet => println!("Something is swimming in {}", object.name)
        }
    }

}


//Convert to trait of u32?
pub struct Zorder;
impl Zorder {

    pub fn to_cell(mut zorder:u32, depth:u32) -> UVec2 {
        let (mut x, mut y) = (0, 0);
        for layer in 0 .. depth {
            x |= (zorder & 0b1) << layer;
            zorder >>= 1;
            y |= (zorder & 0b1) << layer;
            zorder >>= 1;
        }
        UVec2::new(x, y)
    }

    pub fn from_cell(cell:UVec2, depth:u32) -> u32 {
        let mut zorder = 0;
        for layer in (0 .. depth).rev() {
            let step = (((cell.y >> layer) & 0b1) << 1 ) | ((cell.x >> layer) & 0b1);
            zorder = (zorder << 2) | step;
        }
        zorder
    }

    //Don't call if direction.length() == 0
    pub fn from_configured_direction(direction:Vec2, configuration:u8) -> usize {
        let clamped: Vec2 = direction.signum().clamp(Vec2::ZERO, Vec2::ONE);
        if direction.x == 0. {
            2 * clamped.y as usize | if configuration == 0 || configuration == 2 { 1 } else { 0 }
        } else if direction.y == 0. {
            clamped.x as usize | if configuration == 0 || configuration == 1 { 2 } else { 0 }
        } else {
            2 * clamped.y as usize | clamped.x as usize
        }
    }

    pub fn read(zorder:u32, layer:u32, depth:u32) -> u32 {
        zorder >> (2 * (depth - layer)) & 0b11
    }

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



