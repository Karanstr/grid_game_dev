use core::panic;
use std::f32::consts::PI;
use macroquad::prelude::*;

use crate::graph::{Index, NodePointer, Path2D, SparseDirectedGraph};

//Figure out all of the weird return type recasting I'm doing rn

pub struct Object {
    root : NodePointer,
    pub position : Vec2,
    velocity : Vec2,
    rotation : f32, 
    angular_velocity : f32,
    domain : Vec2,
}

impl Object {

    pub fn new(root:NodePointer, position:Vec2, domain:Vec2) -> Self {
        Self {
            root,
            position,
            velocity : Vec2::ZERO,
            rotation : 0.0,
            angular_velocity : 0.,
            domain,
        }
    }

    //This is all raymarching stuff, move to scene?

    pub fn coord_to_cell(&self, point:Vec2, depth:u32) -> [Option<UVec2>; 4] {
        let mut four_points = [None; 4];
        let half_length = self.domain/2.;
        let block_size = self.domain / 2f32.powf(depth as f32);
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
                Some( (cur_point / block_size).floor().as_uvec2() )
            } else { None }
        }
        four_points
    }

    fn calculate_corner(&self, sign:Vec2, grid_cell:UVec2, depth:u32) -> Vec2 {
        let box_size = self.domain / 2u32.pow(depth) as f32;
        let quadrant = (sign + 0.5).abs().floor();
        grid_cell.as_vec2() * box_size + box_size * quadrant + self.position - self.domain/2.
    }

    fn next_intersection(&self, cell:UVec2, depth:u32, position:Vec2, max_displacement:Vec2) -> Option<(Vec2, Vec2)> {
        let corner = self.calculate_corner(max_displacement.signum(), cell, depth);
        let ticks = (corner - position) / max_displacement;
        match ticks.min_element() {
            no_hit if no_hit > 1. => None,
            hitting => {
                let distance = max_displacement * hitting;
                let walls_hit = if ticks.y < ticks.x {
                    Vec2::new(0., 1.)
                } else if ticks.x < ticks.y {
                    Vec2::new(1., 0.)
                } else { Vec2::ONE };
                Some((position + distance, walls_hit))
            }
        }
    }

    fn find_real_node(&self, graph:&SparseDirectedGraph, cell:UVec2, max_depth:u32) -> (NodePointer, UVec2, u32) {
        let bit_path = cell_to_zorder(cell, max_depth);
            match graph.read_destination(self.root, &Path2D::from(bit_path, max_depth as usize)) {
                Ok((cell_pointer, real_depth)) => {
                    let zorder = bit_path as u32 >> 2 * (max_depth - real_depth);
                    (cell_pointer, zorder_to_cell(zorder, real_depth), real_depth)
                },
                Err(error) => panic!("{error:?}")
            }
    }

    fn get_data_at_position(&self, graph:&SparseDirectedGraph, position:Vec2, max_depth:u32) -> [Option<(bool, UVec2, u32)>; 4] {
        let max_depth_cells = self.coord_to_cell(position, max_depth);
        let mut data: [Option<(bool, UVec2, u32)>; 4] = [None; 4];
        for i in 0 .. 4 {
            if let Some(grid_cell) = max_depth_cells[i] {
                let (node, real_cell, depth) = self.find_real_node(graph, grid_cell, max_depth);
                    //If the node is solid (once we have a proper definition of solid abstract this)
                    data[i] = Some((*node.index == 1, real_cell, depth))
            }
        }
        data
    }

    fn next_collision(&self, graph:&SparseDirectedGraph, position:Vec2, displacement:Vec2, max_depth:u32) -> Option<(Vec2, Vec2)> {
        let mut cur_position = position;
        let mut rem_displacement = displacement;
        //I don't think I care which possibility we start with when moving along a single axis
        //I remember using the inverse velocity for the first cell causing problems, but I have no clue why
        let initial = velocity_to_zorder_direction(-displacement);
        let relevant_cells = velocity_to_zorder_direction(displacement);
        let (_, mut grid_cell, mut cur_depth) = self.get_data_at_position(graph, cur_position, max_depth)[initial[0]]?;
        while rem_displacement.length() != 0. {
            let (new_position, walls_hit) = self.next_intersection(grid_cell, cur_depth, cur_position, rem_displacement)?;
            rem_displacement -= new_position - cur_position;
            cur_position = new_position;
            let data = self.get_data_at_position(graph, cur_position, max_depth);
            let mut hit_count = 0;
            for possibility in relevant_cells.iter() {
                if let Some((solid, cell, depth)) = data[*possibility] {
                    if solid { hit_count += 1 } else { grid_cell = cell; cur_depth = depth}
                }
            }
            if hit_count == relevant_cells.len() { return Some((cur_position, walls_hit)) }
        }
        None
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

pub struct Scene {
    pub graph : SparseDirectedGraph,
}

impl Scene {

    pub fn new() -> Self {
        Self {
            graph : SparseDirectedGraph::new(),
        }
    }

    pub fn render(&self, object:&Object, draw_lines:bool) {
        let filled_blocks = self.graph.dfs_leaves(object.root);
        for (zorder, depth, index) in filled_blocks {
            let block_domain = object.domain / 2u32.pow(depth) as f32;
            let grid_cell = zorder_to_cell(zorder, depth);
            let offset = Vec2::new(grid_cell.x as f32, grid_cell.y as f32) * block_domain + object.position - object.domain/2.;
            let color = if *index == 0 { BLACK } else if *index == 1 { MAROON } else { WHITE };
            draw_rect(offset, block_domain, color);
            if draw_lines { outline_rect(offset, block_domain, 1., WHITE) }
        }
    }

    pub fn move_with_collisions(&mut self, moving:&mut Object, hitting:&Object) {
        if moving.velocity.length() != 0. {
            let mut cur_position = moving.position;
            let mut remaining_displacement = moving.velocity;
            while remaining_displacement.length() != 0. {
                match hitting.next_collision(&self.graph, cur_position, remaining_displacement, 5) {
                    None => break,
                    Some((new_position, walls_hit)) => {
                        remaining_displacement -= new_position - cur_position;
                        remaining_displacement *= (walls_hit - 1.).abs();
                        moving.velocity *= (walls_hit - 1.).abs();
                        cur_position = new_position;
                    }
                }
            }
            moving.position = cur_position + remaining_displacement;
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

    pub fn set_cell_with_mouse(&mut self, modified:&mut Object, mouse_pos:Vec2, depth:u32, color:Color) {
        let block_size = modified.domain / 2u32.pow(depth) as f32;

        let rel_mouse_pos = mouse_pos - modified.position;
        let unrounded_cell = rel_mouse_pos / block_size;
        let mut edit_cell = IVec2::new(
            round_away_0_pref_pos(unrounded_cell.x),
            round_away_0_pref_pos(unrounded_cell.y)
        );
        let blocks_on_half = 2i32.pow(depth - 1);
        if edit_cell.abs().max_element() > blocks_on_half { return }
        edit_cell += blocks_on_half;
        if edit_cell.x > blocks_on_half { edit_cell.x -= 1 }
        if edit_cell.y > blocks_on_half { edit_cell.y -= 1 }
        let bit_path = cell_to_zorder(edit_cell.as_uvec2(), depth);
        let path = Path2D::from(bit_path, depth as usize);

        let child_index = Index( match color {
            MAROON => 1,
            BLACK => 0,
            _ => 2,
        } );

        let new_child = NodePointer::new(child_index, 0b0000);

        if let Ok(root) = self.graph.set_node_child(modified.root, &path, new_child) {
            modified.root = root
        };
    }



}


fn velocity_to_zorder_direction(velocity:Vec2) -> Vec<usize> {
    let velocity_dir = velocity.signum();
    let clamped = velocity_dir.clamp(Vec2::ZERO, Vec2::ONE).as_uvec2();
    if velocity_dir.x == 0. && velocity_dir.y == 0. {
        vec![0b00, 0b01, 0b10, 0b11]
    } else if velocity_dir.x == 0. {
        vec![
            2 * clamped.y as usize | 1, 
            2 * clamped.y as usize | 0,
        ]
    } else if velocity_dir.y == 0. {
        vec![
            clamped.x as usize, 
            2 | clamped.x as usize,
        ]
    } else {
        vec![2 * clamped.y as usize | clamped.x as usize] 
    }
}

//Will overflow if our z-order goes 16 layers deep. So.. don't do that
fn zorder_to_cell(mut zorder:u32, depth:u32) -> UVec2 {
    let (mut x, mut y) = (0, 0);
    for layer in 0 .. depth {
        x |= (zorder & 0b1) << layer;
        zorder >>= 1;
        y |= (zorder & 0b1) << layer;
        zorder >>= 1;
    }
    UVec2::new(x, y)
}

//Figure this out later
fn cell_to_zorder(grid_cell:UVec2, root_layer:u32) -> usize {
    let mut cell = 0;
    for layer in (0 ..= root_layer).rev() {
        let step = (((grid_cell.y >> layer) & 0b1) << 1 ) | ((grid_cell.x >> layer) & 0b1);
        cell = (cell << 2) | step;
    }
    cell as usize
}

//Figure out where to put these
fn round_away_0_pref_pos(number:f32) -> i32 {
    if number < 0. {
        number.floor() as i32
    } else if number > 0. {
        number.ceil() as i32
    }
    else {
        //We don't want to return 0 when we're trying to avoid 0
        //And the name of the function is prefer_position, so..
        1 
    }
}

#[allow(dead_code)]
mod vec_friendly_drawing {
    use macroquad::prelude::*;

    pub fn draw_rect(top_left_corner:Vec2, length:Vec2, color:Color) {
        draw_rectangle(top_left_corner.x, top_left_corner.y, length.y, length.x, color);
    }

    pub fn draw_centered_rect(position:Vec2, length:Vec2, color:Color) {
        let real_pos = position - length/2.;
        draw_rectangle(real_pos.x, real_pos.y, length.y, length.x, color);
    }

    pub fn outline_rect(position:Vec2, length:Vec2, line_width:f32, color:Color) {
        draw_rectangle_lines(position.x, position.y, length.x, length.x, line_width, color);
    }

    pub fn draw_vec_line(point1:Vec2, point2:Vec2, line_width:f32, color:Color) {
        draw_line(point1.x, point1.y, point2.x, point2.y, line_width, color);
    }

}



