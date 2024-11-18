use core::panic;
use std::f32::consts::PI;
use macroquad::{prelude::*, telemetry::frame};

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

    //This is all raymarching stuff

    pub fn coord_to_cartesian(&self, point:Vec2, depth:u32) -> [Option<UVec2>; 4] {
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

    fn step_towards_corner(&self, corner:Vec2, start:Vec2, velocity:Vec2) -> Vec2 {
        //Yeah I divide by 0. No I don't care
        (corner-start)/velocity
    }

    fn calculate_corner(&self, vel_sign:Vec2, cartesian_cell:UVec2, depth:u32) -> Vec2 {
        let box_size = self.domain / 2u32.pow(depth) as f32;
        let quadrant = (vel_sign + 0.5).abs().floor();
        cartesian_cell.as_vec2() * box_size + box_size * quadrant + self.position - self.domain/2.
    }

    fn find_real_cell(&self, graph:&SparseDirectedGraph, cell:Option<UVec2>, max_depth:u32) -> Option<(NodePointer, UVec2, u32)> {
        let cell_zorder = cartesian_to_zorder(cell?, max_depth);
            match graph.read_destination(self.root, &Path2D::from(cell_zorder, max_depth as usize)) {
                Ok((cell_data, real_depth)) => {
                    let culled_zorder = cell_zorder as u32 >> 2 * (max_depth - real_depth);
                    Some((cell_data, zorder_to_cartesian(culled_zorder, real_depth), real_depth))
                },
                Err(error) => {
                    dbg!(error);
                    None
                }
            }
    }

    fn march(&self, graph:&SparseDirectedGraph, moving:&Object, max_depth:u32) -> (Vec2, Vec2) {
        let mut step_count = 0;
        let mut cur_position = moving.position;
        let mut rem_velocity = moving.velocity;
        let mut collision_vel_modifier = Vec2::ONE;
        let coming_from = velocity_to_zorder_direction(rem_velocity * -1.);
        let start_cell = self.coord_to_cartesian(cur_position, max_depth)[coming_from];
        let (mut cell_cartesian, mut cell_depth) = match self.find_real_cell(graph, start_cell, max_depth) {
            Some((_, cartesian, depth)) => (cartesian, depth),
            None => {
                //Figure out how to get corner from the exterior, until then:
                println!("You're outside right now. Stop that");
                return (cur_position + rem_velocity, moving.velocity)
            }
        };
        let going_to = velocity_to_zorder_direction(rem_velocity);
        loop {
            step_count += 1;
            rem_velocity *= collision_vel_modifier;
            let frames_to_hit = {
                let corner = self.calculate_corner(rem_velocity.signum(), cell_cartesian, cell_depth);
                self.step_towards_corner(corner, cur_position, rem_velocity)
            };
            match frames_to_hit.min_element() {
                no_hit if no_hit >= 1. => return (cur_position + rem_velocity, moving.velocity * collision_vel_modifier),
                hitting => {
                    let traveled = rem_velocity * hitting;
                    rem_velocity -= traveled;
                    cur_position += traveled;
                    let new_cell = self.coord_to_cartesian(cur_position, max_depth)[going_to];
                    match self.find_real_cell(graph, new_cell, max_depth) {
                        Some((cell_data, cell, depth)) => {
                            cell_cartesian = cell;
                            cell_depth = depth;
                            if *cell_data.index == 1 {
                                if frames_to_hit.x == frames_to_hit.y {
                                    collision_vel_modifier = Vec2::ZERO
                                } else if frames_to_hit.y < frames_to_hit.x {
                                    collision_vel_modifier.y = 0.
                                } else if frames_to_hit.x < frames_to_hit.y {
                                    collision_vel_modifier.x = 0.
                                }
                            }
                        },
                        None => {
                            //Figure out how to get corner from the exterior, until then:
                            println!("You're outside right now. Stop that");
                            return (cur_position + rem_velocity, moving.velocity)
                        }
                    }
                }
            }
            if step_count == 10 {
                panic!();
            }
        }
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
            let cartesian_cell = zorder_to_cartesian(zorder, depth);
            let offset = Vec2::new(cartesian_cell.x as f32, cartesian_cell.y as f32) * block_domain + object.position - object.domain/2.;
            let color = if *index == 0 { BLACK } else if *index == 1 { MAROON } else { WHITE };
            draw_rect(offset, block_domain, color);
            if draw_lines {
                outline_rect(offset, block_domain, 1., WHITE);
            }
        }
    }

    pub fn move_with_collisions(&mut self, moving:&mut Object, hitting:&Object) {
        if moving.velocity.length() != 0. {
            (moving.position, moving.velocity) = hitting.march(&self.graph, moving, 5);
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
        let cell = cartesian_to_zorder(edit_cell.as_uvec2(), depth);
        let path = Path2D::from(cell, depth as usize);

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


fn velocity_to_zorder_direction(velocity:Vec2) -> usize {
    let velocity_dir = velocity.signum().clamp(Vec2::ZERO, Vec2::ONE);
    1 * velocity_dir.x as usize | 2 * velocity_dir.y as usize
}

//Will overflow if our z-order goes 32 layers deep. So.. don't do that
fn zorder_to_cartesian(mut zorder:u32, depth:u32) -> UVec2 {
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
fn cartesian_to_zorder(cartesian_cell:UVec2, root_layer:u32) -> usize {
    let mut cell = 0;
    for layer in (0 ..= root_layer).rev() {
        let step = (((cartesian_cell.y >> layer) & 0b1) << 1 ) | ((cartesian_cell.x >> layer) & 0b1);
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



