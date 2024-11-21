use std::f32::consts::PI;
use macroquad::prelude::*;

use crate::graph::{Index, NodePointer, Path2D, SparseDirectedGraph};

#[derive(Clone, Copy, Debug, PartialEq)]
enum OnTouch {
    Ignore,
    Resist(IVec2),
    //...
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum BlockType {
    Air,
    Ground,
    //...
}

pub struct Object {
    root : NodePointer,
    position : Vec2,
    velocity : Vec2,
    rotation : f32, // >:( iykyk
    angular_velocity : f32,
    grid_length : f32,
}

impl Object {

    pub fn new(root:NodePointer, position:Vec2, grid_length:f32) -> Self {
        Self {
            root,
            position,
            velocity : Vec2::ZERO,
            rotation : 0.0,
            angular_velocity : 0.,
            grid_length,
        }
    }

    fn cell_length(&self, depth:u32) -> f32 {
        self.grid_length / 2f32.powf(depth as f32)
    }

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

    fn next_intersection(&self, cell:UVec2, depth:u32, position:Vec2, velocity:Vec2) -> (Vec2, f32, IVec2) {
        let cell_length = self.cell_length(depth);
        let quadrant = (velocity.signum() + 0.5).abs().floor();
        let corner = cell.as_vec2() * cell_length + cell_length * quadrant + self.position - self.grid_length/2.;
        let ticks = (corner - position) / velocity;
        let ticks_to_first_hit = ticks.min_element();
        let walls_hit = if ticks.y < ticks.x {
            IVec2::new(0, 1)
        } else if ticks.x < ticks.y {
            IVec2::new(1, 0)
        } else { IVec2::ONE };
        (position + velocity * ticks_to_first_hit, ticks_to_first_hit, walls_hit)
    }

    //Add a class for this return and slide_check's input?
    fn find_real_node(&self, graph:&SparseDirectedGraph, cell:UVec2, max_depth:u32) -> (NodePointer, UVec2, u32) {
        let bit_path = zorder_from_cell(cell, max_depth);
            match graph.read_destination(self.root, &Path2D::from(bit_path, max_depth as usize)) {
                Ok((cell_pointer, real_depth)) => {
                    dbg!(real_depth);
                    let zorder = bit_path as u32 >> 2 * (max_depth - real_depth);
                    (cell_pointer, zorder_to_cell(zorder, real_depth), real_depth)
                },
                Err(error) => panic!("{error:?}")
            }
    }

    fn on_touch(&self, block:BlockType) -> OnTouch {
        match block {
            BlockType::Air => OnTouch::Ignore,
            BlockType::Ground => OnTouch::Resist(IVec2::ZERO),
            _ => OnTouch::Ignore,
        }
    }

    fn block_type(&self, index:Index) -> BlockType {
        match *index {
            0 => BlockType::Air,
            1 => BlockType::Ground,
            _ => BlockType::Air,
        }
    }

    fn get_data_at_position(&self, graph:&SparseDirectedGraph, position:Vec2, max_depth:u32) -> [Option<(BlockType, UVec2, u32)>; 4] {
        let max_depth_cells = self.coord_to_cell(position, max_depth);
        let mut data: [Option<(BlockType, UVec2, u32)>; 4] = [None; 4];
        for i in 0 .. 4 {
            if let Some(grid_cell) = max_depth_cells[i] {
                let (node, real_cell, depth) = self.find_real_node(graph, grid_cell, max_depth);
                data[i] = Some((self.block_type(node.index), real_cell, depth))
            }
        }
        data
    }

    fn slide_check(&self, walls_hit:IVec2, displacement:Vec2, position_data:[Option<(BlockType, UVec2, u32)>; 4]) -> IVec2 {
        let mut updated_walls = walls_hit;
        let (x_slide_check, y_slide_check) = if displacement.x < 0. && displacement.y < 0. { //(-,-)
            (2, 1)
        } else if displacement.x < 0. && displacement.y > 0. { //(-,+)
            (0, 3)
        } else if displacement.x > 0. && displacement.y < 0. { //(+,-)
            (3, 0)
        } else { //(+,+)
            (1, 2)
        };
        let x_block_collision = match position_data[x_slide_check] {
            Some((block, ..)) => self.on_touch(block),
            None => OnTouch::Resist(IVec2::ZERO)
        };
        let y_block_collision = match position_data[y_slide_check] {
            Some((block, ..)) => self.on_touch(block),
            None => OnTouch::Resist(IVec2::ZERO)
        };
        if x_block_collision != y_block_collision {
            if let OnTouch::Resist(_) = x_block_collision { updated_walls.x = 0 }
            if let OnTouch::Resist(_) = y_block_collision { updated_walls.y = 0 }
        }
        updated_walls
    }

    //Add the option for no orientation, making my life difficult in exchange for functionality
    fn next_boundary(&self, graph:&SparseDirectedGraph, position:Vec2, displacement:Vec2, orientation:u8, max_depth:u32, first:bool) -> Option<(Vec2, OnTouch)> {
        let relevant_cell = velocity_to_zorder_direction(displacement, orientation);
        let mut cur_position = position;
        let mut rem_displacement = displacement;
        let (cur_block_type, mut grid_cell, mut cur_depth) = self.get_data_at_position(graph, position, max_depth)[if first { velocity_to_zorder_direction(-displacement, orientation) } else {relevant_cell}]?;
        loop {
            let (new_position, ticks_to_reach, walls_hit) = self.next_intersection(grid_cell, cur_depth, cur_position, rem_displacement);
            if ticks_to_reach >= 1. { return None }
            let delta = new_position - cur_position;
            rem_displacement -= delta;
            cur_position = new_position;
            let data = self.get_data_at_position(graph, new_position, max_depth);
            let new_block_type;
            (new_block_type, grid_cell, cur_depth) = data[relevant_cell]?;
            if ticks_to_reach == 0. && new_block_type == cur_block_type { continue }
            return match self.on_touch(new_block_type) {
                OnTouch::Ignore => {
                    Some((new_position, OnTouch::Ignore))
                },
                OnTouch::Resist(_) => {
                    if walls_hit.x == walls_hit.y {
                        Some((new_position, OnTouch::Resist(self.slide_check(walls_hit, rem_displacement, data))))
                    } else {
                        Some((new_position, OnTouch::Resist(walls_hit)))
                    }
                },
            }
        }
    }
    
    fn all_collisions(&self, graph:&SparseDirectedGraph, position:Vec2, velocity:Vec2, max_depth:u32) -> (Vec2, Vec2) {
        let mut cur_position = position;
        let mut remaining_displacement = velocity;
        let mut end_velocity = velocity;
        let mut first = true;
        while remaining_displacement.length() != 0. {
            match self.next_boundary(graph, cur_position, remaining_displacement, 0, max_depth, first) {
                Some((new_position, action)) => {
                    remaining_displacement -= new_position - cur_position;
                    cur_position = new_position;
                    match action {
                        OnTouch::Ignore => first = false,
                        OnTouch::Resist(walls_hit) => {
                            first = true;
                            if walls_hit.x == 1 {
                                remaining_displacement.x = 0.;
                                end_velocity.x = 0.;
                            }
                            if walls_hit.y == 1 {
                                remaining_displacement.y = 0.;
                                end_velocity.y = 0.;
                            }
                        }
                    }
                },
                None => break,
            }
        }
        (cur_position + remaining_displacement, end_velocity)
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
            let cell_length = object.cell_length(depth);
            let grid_cell = zorder_to_cell(zorder, depth);
            let offset = grid_cell.as_vec2() * cell_length + object.position - object.grid_length/2.;
            let color = if *index == 0 { BLACK } else if *index == 1 { MAROON } else { WHITE };
            draw_square(offset, cell_length, color);
            if draw_lines { outline_square(offset, cell_length, 1., WHITE) }
        }
    }

    pub fn move_with_collisions(&mut self, moving:&mut Object, hitting:&Object) {
        if moving.velocity.length() != 0. {
            (moving.position, moving.velocity) = hitting.all_collisions(&self.graph, moving.position, moving.velocity, 5);
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
        let block_size = modified.cell_length(depth);

        let rel_mouse_pos = mouse_pos - modified.position;
        let unrounded_cell = rel_mouse_pos / block_size;
        let mut edit_cell = IVec2::new(
            round_away_0_pref_pos(unrounded_cell.x),
            round_away_0_pref_pos(unrounded_cell.y)
        );
        let blocks_on_half = 2i32.pow(depth);
        if edit_cell.abs().max_element() > blocks_on_half { return }
        edit_cell += blocks_on_half;
        if edit_cell.x > blocks_on_half { edit_cell.x -= 1 }
        if edit_cell.y > blocks_on_half { edit_cell.y -= 1 }
        let bit_path = zorder_from_cell(edit_cell.as_uvec2(), depth);
        let path = Path2D::from(bit_path, depth as usize);

        let child_index = Index( match color {
            MAROON => 1,
            BLACK => 0,
            _ => 0
        } );

        let new_child = NodePointer::new(child_index, 0b0000);

        if let Ok(root) = self.graph.set_node_child(modified.root, &path, new_child) {
            modified.root = root
        };
    }



}

//Don't call if velocity.length() == 0
fn velocity_to_zorder_direction(velocity:Vec2, orientation:u8) -> usize {
    let clamped = velocity.signum().clamp(Vec2::ZERO, Vec2::ONE);
    if velocity.x == 0. {
        2 * clamped.y as usize | if orientation == 0 || orientation == 2 { 1 } else { 0 }
    } else if velocity.y == 0. {
        clamped.x as usize | if orientation == 0 || orientation == 1 { 2 } else { 0 }
    } else {
        2 * clamped.y as usize | clamped.x as usize
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

fn zorder_from_cell(grid_cell:UVec2, root_layer:u32) -> usize {
    let mut cell = 0;
    for layer in (0 ..= root_layer).rev() {
        let step = (((grid_cell.y >> layer) & 0b1) << 1 ) | ((grid_cell.x >> layer) & 0b1);
        cell = (cell << 2) | step;
    }
    cell as usize
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



