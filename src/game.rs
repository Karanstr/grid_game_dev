use std::f32::consts::PI;
use std::collections::VecDeque;
use macroquad::prelude::*;
use crate::graph::{NodePointer, SparseDirectedGraph, Zorder};
pub use crate::graph::Index;
//Clean up this import stuff
mod collision_utils;
use collision_utils::*;

pub struct Object {
    pub root : NodePointer,
    position : Vec2,
    grid_length : f32,
    velocity : Vec2,
    rotation : f32,
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

    fn cell_top_left_corner(&self, cell:UVec2, depth:u32) -> Vec2 {
        let cell_length = self.cell_length(depth);
        cell.as_vec2() * cell_length + self.position - self.grid_length/2.
    }

    //Change to relative position?
    fn coord_to_cell(&self, point:Vec2, depth:u32) -> [Option<UVec2>; 4] {
        let mut four_points = [None; 4];
        let half_length = self.grid_length/2.;
        let cell_length = self.cell_length(depth);
        let offset = 0.01;
        for i in 0 .. 4 {
            let direction = Vec2::new(
                if i & 0b1 == 1 { 1. } else { -1. },
                if i & 0b10 == 0b10 { 1. } else { -1. }
            );
            let top_left = self.position - half_length;
            let cur_point = point - top_left + offset * direction;
            four_points[i] = if cur_point.clamp(Vec2::ZERO, Vec2::splat(self.grid_length)) == cur_point {
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

    fn bound_check(&self, position:Vec2) -> (BVec2, BVec2) {
        let top_left = self.position - self.grid_length/2.;
        let bottom_right = self.position + self.grid_length/2.;
        (
            BVec2::new(
                if position.x < top_left.x || position.x > bottom_right.x { false } else { true },
                if position.y < top_left.y || position.y > bottom_right.y { false } else { true }
            ),
            BVec2::new(
                if position.x == top_left.x || position.x == bottom_right.x { true } else { false },
                if position.y == top_left.y || position.y == bottom_right.y { true } else { false }
            )
        )
    }

    fn get_aabb_corners(&self, cur_pos:Vec2, cur_vel:Vec2) -> Vec<Particle> {
        let half_length = self.grid_length/2.;
        Vec::from([
            Particle::new(
                cur_pos + Vec2::new(-half_length, -half_length),
                cur_vel,
                Configurations::TopLeft
            ),
            Particle::new(
                cur_pos + Vec2::new(-half_length, half_length),
                cur_vel,
                Configurations::BottomLeft
            ),
            Particle::new(
                cur_pos + Vec2::new(half_length, -half_length),
                cur_vel,
                Configurations::TopRight
            ),
            Particle::new(
                cur_pos + Vec2::new(half_length, half_length),
                cur_vel,
                Configurations::BottomRight
            ),
        ])
    }

    pub fn apply_linear_force(&mut self, force:Vec2) {
        self.velocity += force;
        self.remove_neglible_vel()
    }

    pub fn apply_forward_force(&mut self, force:Vec2) {
        self.apply_linear_force(force * Vec2::from_angle(self.rotation));
    }

    fn remove_neglible_vel(&mut self) {
        let speed_min = 0.005;
        if self.velocity.x.abs() < speed_min { self.velocity.x = 0. }
        if self.velocity.y.abs() < speed_min { self.velocity.y = 0. }

    }

    pub fn apply_rotational_force(&mut self, torque:f32) {
        self.angular_velocity += torque
    }

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
    pub points_to_draw : Vec<(Vec2, Color, i32)>
}

impl World {

    pub fn new() -> Self {
        Self {
            blocks : BlockPallete::new(),
            graph : SparseDirectedGraph::new(8),
            points_to_draw : Vec::new(),
        }
    }

    pub fn render_cache(&mut self) {
        let mut new_points = Vec::new();
        for (point, color, time) in self.points_to_draw.iter_mut() {
            draw_centered_square(*point, 10., *color);
            let new_time = *time - 1;
            if new_time != 0 {
                new_points.push((*point, *color, new_time))
            }
        }
        self.points_to_draw = new_points;
    }

    fn push_to_render_cache(&mut self, point:Vec2, color:Color, ticks:i32) {
        self.points_to_draw.push((point, color, ticks));
    }

    pub fn render(&self, object:&mut Object, draw_lines:bool) {
        let blocks = self.graph.dfs_leaves(object.root);
        for (zorder, depth, index) in blocks {
            match self.index_color(index) {
                Some(color) => {
                    let top_left_corner = object.cell_top_left_corner(Zorder::to_cell(zorder, depth), depth);
                    draw_square(top_left_corner, object.cell_length(depth), color);
                    if draw_lines { outline_square(top_left_corner, object.cell_length(depth), 2., WHITE) }
                }
                None => { eprintln!("Failed to draw {}, unregistered block", *index) }
            }
        }
    }

    fn cull_and_fill_corners(&self, hitting:&Object, unculled_corners:Vec<Particle>, max_depth:u32) -> VecDeque<Particle> {
        let mut corners = VecDeque::new();
        for corner in 0 .. unculled_corners.len() {
            if unculled_corners[corner].hittable_walls() == BVec2::FALSE { continue }
            let mut culled_corner = unculled_corners[corner].clone();
            culled_corner.position_data = hitting.get_data_at_position(&self, unculled_corners[corner].position, max_depth)[Zorder::from_configured_direction(-unculled_corners[corner].rem_displacement, unculled_corners[corner].configuration)];
            corners.push_back(culled_corner);
        }
        corners
    }

    fn find_next_action(&mut self, moving:&Object, hitting:&Object, cur_pos: Vec2, cur_vel: Vec2, max_depth:u32) -> (OnTouch, Vec2) {
        let mut corners = self.cull_and_fill_corners(hitting, moving.get_aabb_corners(cur_pos, cur_vel), max_depth);  
        let mut vel_left_when_action = Vec2::ZERO;
        let mut action = OnTouch::Ignore;
        while let Some(mut cur_corner) = corners.pop_front() {
            if cur_corner.rem_displacement.length() <= vel_left_when_action.length() { break }

            let hit_point = match self.next_intersection(&cur_corner, hitting, cur_corner.position_data) {
                Some(hit_point) if hit_point.ticks_to_hit < 1. => { hit_point }
                _ => { continue }
            };

            let position_data = hitting.get_data_at_position(&self, hit_point.position, max_depth);
            cur_corner.move_to(hit_point.position, position_data);

            if let Some(data) = cur_corner.position_data {
                match self.index_collision(data.node_pointer.index) {
                    Some(OnTouch::Ignore) => { }
                    Some(OnTouch::Resist(possibly_hit_walls)) => {
                        let hit_walls = possibly_hit_walls & cur_corner.hittable_walls();
                        let checked_walls = { 
                            if hit_walls == BVec2::TRUE {
                                self.slide_check(&cur_corner, position_data)
                            } else { 
                                hit_walls 
                            }
                        };
                        if checked_walls != BVec2::FALSE {
                            action = OnTouch::Resist(
                                if checked_walls != BVec2::TRUE { checked_walls } else { cur_corner.mag_slide_check() }
                            );
                            vel_left_when_action = cur_corner.rem_displacement;
                            continue
                        }
                    } 
                    None => { eprintln!("Attempting to touch {}, an unregistered block!", *data.node_pointer.index); }
                }
            }
            let mut index = corners.len();
            for corner in corners.iter().rev() {
                if corner.rem_displacement.length() >= cur_corner.rem_displacement.length() { break }
                index -= 1;
            }
            corners.insert(index, cur_corner);
        }
        (action, vel_left_when_action)
    }

    pub fn move_with_collisions(&mut self, moving:&mut Object, hitting:&Object, max_depth:u32) {
        if moving.velocity.length() != 0. {
            let mut cur_pos = moving.position;
            let mut cur_vel = moving.velocity;
            let mut modifier = Vec2::ONE;
            //Find all actions
            while cur_vel.length() > 0. {
                let (next_action, remaining_vel) = self.find_next_action(moving, hitting, cur_pos, cur_vel, max_depth);
                cur_pos += cur_vel - remaining_vel;
                cur_vel = remaining_vel;
                match next_action {
                    OnTouch::Ignore => {}
                    OnTouch::Resist(walls_hit) => {
                        if walls_hit.x { 
                            cur_vel.x = 0.;
                            modifier.x *= 0.;
                        }
                        if walls_hit.y { 
                            cur_vel.y = 0.;
                            modifier.y *= 0.;
                        }
                    }
                }
            }
            moving.position = cur_pos;
            moving.velocity *= modifier;
            let drag_multiplier = -0.01;
            moving.apply_linear_force(moving.velocity * drag_multiplier);
        }
        moving.rotation += moving.angular_velocity;
        moving.rotation %= 2.*PI;
        moving.angular_velocity = 0.;
    }

    //Eventually remove all these &Objects, a particle should march through the world hitting any objects in it's path.
    fn next_intersection(&mut self, particle:&Particle, object:&Object, pos_data:Option<LimPositionData>) -> Option<HitPoint> {  
        let half_length = object.grid_length/2.;
        let (within_bounds, on_bounds) = object.bound_check(particle.position);
        let boundary_corner = match pos_data {
            Some(data) => {
                let cell_length = object.cell_length(data.depth);
                let quadrant = (particle.rem_displacement.signum() + 0.5).abs().floor();
                data.cell.as_vec2() * cell_length + cell_length * quadrant + object.position - half_length
            }
            None => { object.position + half_length * -particle.rem_displacement.signum() }
        };
        let ticks = ((boundary_corner - particle.position) / particle.rem_displacement).abs();
        let ticks_to_hit = if within_bounds.x ^ within_bounds.y && ticks.min_element() == 0. || (!within_bounds.x && !within_bounds.y) {
            ticks.max_element()
        } else if on_bounds.x && on_bounds.y && particle.hittable_walls() != BVec2::TRUE {
            return None
        } else { 
            ticks.min_element()
        };
        if ticks_to_hit.is_nan() || ticks_to_hit.is_infinite() { return None } 
        self.push_to_render_cache(particle.position + particle.rem_displacement * ticks_to_hit, RED, 5);
        Some(HitPoint {
            position : particle.position + particle.rem_displacement * ticks_to_hit, 
            ticks_to_hit, 
        })

    }

    fn slide_check(&self, particle:&Particle, position_data:[Option<LimPositionData>; 4]) -> BVec2 {
        //Formalize this with some zorder arithmatic?
        let (x_slide_check, y_slide_check) = if particle.rem_displacement.x < 0. && particle.rem_displacement.y < 0. { //(-,-)
            (2, 1)
        } else if particle.rem_displacement.x < 0. && particle.rem_displacement.y > 0. { //(-,+)
            (0, 3)
        } else if particle.rem_displacement.x > 0. && particle.rem_displacement.y < 0. { //(+,-)
            (3, 0)
        } else { //(+,+)
            (1, 2)
        };
        let x_block_collision = if let Some(pos_data) = position_data[x_slide_check] {
            self.index_collision(pos_data.node_pointer.index).unwrap_or(OnTouch::Ignore)
        } else { OnTouch::Ignore };
        let y_block_collision = if let Some(pos_data) = position_data[y_slide_check] {
            self.index_collision(pos_data.node_pointer.index).unwrap_or(OnTouch::Ignore)
        } else { OnTouch::Ignore };
        BVec2::new(
            !matches!(y_block_collision, OnTouch::Resist(_)),
            !matches!(x_block_collision, OnTouch::Resist(_)),
        )
    }

    pub fn set_cell_with_mouse(&mut self, modified:&mut Object, mouse_pos:Vec2, depth:u32, index:Index) -> Result<(), String> {
        let shifted_point = mouse_pos - modified.position + modified.grid_length/2.;
        if shifted_point.min_element() <= 0. || shifted_point.max_element() >= modified.grid_length {
            return Err("Attempting to edit beyond object domain".to_owned())
        }
        let cell = (shifted_point / modified.cell_length(depth)).ceil().as_uvec2() - 1;
        let path = Zorder::path( Zorder::from_cell(cell, depth), depth );
        if let Ok(root) = self.graph.set_node(modified.root, &path, NodePointer::new(index)) {
            modified.root = root;
            Ok(())
        } else { Err("Failed to modify cell. Likely means structure is corrupted.".to_owned()) }
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

    //Doesn't currently account for OnTouch::Resist having configurable walls
    pub fn exposed_corners(&self, root:NodePointer, cell_zorder:u32, cell_depth:u32, max_depth:u32) -> u8 {
        let mut exposed_mask = 0b1111;
        let checks = [
            (IVec2::new(-1, 0), Configurations::TopLeft),
            (IVec2::new(0, -1), Configurations::TopLeft),
            (IVec2::new(-1, -1), Configurations::TopLeft),
            (IVec2::new(1, 0), Configurations::TopRight),
            (IVec2::new(0, -1), Configurations::TopRight),
            (IVec2::new(1, -1), Configurations::TopRight),
            (IVec2::new(-1, 0), Configurations::BottomLeft),
            (IVec2::new(0, 1), Configurations::BottomLeft),
            (IVec2::new(-1, 1), Configurations::BottomLeft),
            (IVec2::new(1, 0), Configurations::BottomRight),
            (IVec2::new(0, 1), Configurations::BottomRight),
            (IVec2::new(1, 1), Configurations::BottomRight),
        ];
        for i in 0 .. 4 {
            for j in 0 .. 3 {
                let (offset, config) = checks[i*3 + j];
                let mut check_zorder = {
                    if let Some(zorder) = Zorder::step_cartesianly(cell_zorder, cell_depth, offset) {
                        zorder
                    } else { 
                        continue 
                    }
                };
                for _ in 0 .. max_depth - cell_depth {
                    check_zorder = check_zorder << 2 | (3 - config.to_index()) as u32
                }
                let path = Zorder::path(check_zorder, max_depth);
                let (node_pointer, _) = self.graph.read(root, &path);
                if let Some(OnTouch::Resist(walls)) = self.index_collision(node_pointer.index) {
                    if walls != BVec2::TRUE { continue }
                    exposed_mask -= 1 << i;
                    break
                }
            }
        }
        exposed_mask
    }


}


impl Zorder {
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

