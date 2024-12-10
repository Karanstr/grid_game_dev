use std::f32::consts::PI;
use macroquad::prelude::*;
use crate::graph::{NodePointer, SparseDirectedGraph};
pub use crate::graph::Index;
//Clean up this import stuff
mod collision_utils;
use collision_utils::*;

pub struct Object {
    root : NodePointer,
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

    pub fn set_position(&mut self, new_position:Vec2) {
        self.position = new_position;
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
                    let top_left_corner = object.cell_top_left_corner(Zorder::to_cell(zorder, depth), depth);
                    draw_square(top_left_corner, object.cell_length(depth), color);
                    if draw_lines { outline_square(top_left_corner, object.cell_length(depth), 2., WHITE) }
                }
                None => { eprintln!("Failed to draw {}, unregistered block", *index) }
            }
        }
    }

    //Eventually write actual code for this
    pub fn get_corners(&self, object:&Object, cur_pos:Vec2, cur_vel:Vec2) -> Vec<(Particle, Option<LimPositionData>)> {
        let half_length = object.grid_length/2.;
        Vec::from([
            (Particle{
                position : cur_pos + Vec2::new(-half_length, -half_length), 
                velocity: cur_vel, 
                configuration : Configurations::TopLeft
            }, None ),
            (Particle{
                position : cur_pos + Vec2::new(half_length, -half_length), 
                velocity: cur_vel, 
                configuration : Configurations::TopRight
            }, None ),
            (Particle{
                position : cur_pos + Vec2::new(-half_length, half_length), 
                velocity: cur_vel, 
                configuration : Configurations::BottomLeft
            }, None ),
            (Particle{
                position : cur_pos + Vec2::new(half_length, half_length), 
                velocity: cur_vel, 
                configuration : Configurations::BottomRight
            }, None ),
        ])
    }

    pub fn move_with_collisions(&mut self, moving:&mut Object, hitting:&Object, max_depth:u32) {
        if moving.velocity.length() != 0. {
            let mut cur_pos = moving.position;
            let mut cur_vel = moving.velocity;
            let mut all_walls_hit = IVec2::ZERO;
            //Find all collisions
            loop {
                let mut corners = self.get_corners(moving, cur_pos, cur_vel);
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
                    let hit_point = match self.next_intersection(cur_point, hitting, *cur_pos_data) {
                        Some(hit) => {
                            draw_centered_square(hit.position, 5., ORANGE);
                            if hit.ticks_to_hit >= 1. { 
                                corners.swap_remove(cur_corner_index); 
                                continue
                            };
                            hit
                        }
                        None => {
                            corners.swap_remove(cur_corner_index); 
                            continue
                        }
                    };
                    let position_data = hitting.get_data_at_position(&self, hit_point.position, max_depth);
                    *cur_pos_data = position_data[Zorder::from_configured_direction(cur_point.velocity, cur_point.configuration)];
                    cur_point.velocity -= hit_point.position - cur_point.position;
                    cur_point.position = hit_point.position;
                    if let Some(data) = cur_pos_data {
                        match self.index_collision(data.node_pointer.index) {
                            Some(OnTouch::Ignore) => {}
                            Some(OnTouch::Resist(possibly_hit_walls)) => {
                                match possibly_hit_walls * cur_point.hittable_walls() * self.slide_check(&cur_point, position_data) {
                                    no_walls_hit if no_walls_hit == IVec2::ZERO => { 
                                        if let Configurations::TopLeft = cur_point.configuration {
                                            dbg!(self.slide_check(&cur_point, position_data));
                                            dbg!(&cur_point);
                                        }
                                        continue 
                                    }
                                    some_walls_hit => {
                                        walls_hit = some_walls_hit;
                                        vel_left_when_hit = cur_point.velocity;
                                        corners.swap_remove(cur_corner_index);
                                    }
                                }
                            }
                            None => { eprintln!("Attempting to touch {}, an unregistered block!", *data.node_pointer.index); }
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
            moving.set_position(cur_pos);
            //Make setters for this instead of directly assigning?
            if all_walls_hit.x == 1 { moving.velocity.x = 0. }
            if all_walls_hit.y == 1 { moving.velocity.y = 0. }
            let drag_multiplier = -0.01;
            moving.apply_linear_force(moving.velocity * drag_multiplier);
        }
        moving.rotation += moving.angular_velocity;
        moving.rotation %= 2.*PI;
        moving.angular_velocity = 0.;
    }

    //Eventually remove all these &Objects, a particle should march through the world hitting any objects in it's path.
    pub fn next_intersection(&self, particle:&Particle, object:&Object, pos_data:Option<LimPositionData>) -> Option<HitPoint> {  
        let half_length = object.grid_length/2.;
        let (within_bounds, on_bounds) = object.bound_check(particle.position);
        let boundary_corner = match pos_data {
            Some(data) => {
                let cell_length = object.cell_length(data.depth);
                let quadrant = (particle.velocity.signum() + 0.5).abs().floor();
                data.cell.as_vec2() * cell_length + cell_length * quadrant + object.position - half_length
            }
            None => {
                let quadrant = particle.velocity.signum();
                object.position + half_length * -quadrant
            }
        };
        let ticks = ((boundary_corner - particle.position) / particle.velocity).abs();
        let ticks_to_hit = if within_bounds.x ^ within_bounds.y && ticks.min_element() == 0. {
            ticks.max_element()
        } else if on_bounds.x && on_bounds.y && particle.hittable_walls() != IVec2::ONE {
            return None
        } else {
            ticks.min_element() 
        };

        if ticks_to_hit.is_nan() || ticks_to_hit.is_infinite() {
            None
        } else {
            Some(HitPoint {
                position : particle.position + particle.velocity * ticks_to_hit, 
                ticks_to_hit, 
            }) 
        }
    }

    fn slide_check(&self, particle:&Particle, position_data:[Option<LimPositionData>; 4]) -> IVec2 {
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
            self.index_collision(pos_data.node_pointer.index).unwrap_or(OnTouch::Ignore)
        } else { OnTouch::Ignore };
        let y_block_collision = if let Some(pos_data) = position_data[y_slide_check] {
            self.index_collision(pos_data.node_pointer.index).unwrap_or(OnTouch::Ignore)
        } else { OnTouch::Ignore };
        if x_block_collision == y_block_collision {
            if let OnTouch::Resist(_) = x_block_collision {
                return IVec2::ONE
            }
        }
        IVec2::new(
            if let OnTouch::Resist(_) = y_block_collision { 0 } else { 1 },
            if let OnTouch::Resist(_) = x_block_collision { 0 } else { 1 },
        )
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
        //and the name of the function is prefer_positive, so..
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



