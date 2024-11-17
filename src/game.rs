use std::f32::consts::PI;
use macroquad::prelude::*;

use crate::graph::{NodePointer, SparseDirectedGraph, Path2D, Index};

pub struct Object {
    root : NodePointer,
    position : Vec2,
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

    fn coord_to_cartesian(&self, point:Vec2, depth:u32) -> [Option<IVec2>; 4] {
        let mut four_points: [Option<IVec2>; 4] = [None; 4];
        let half_length = self.domain/2.;
        let block_size = self.domain / 2f32.powf(depth as f32);
        let top_left = self.position - half_length;
        let bottom_right = self.position + half_length;
        if point.clamp(top_left, bottom_right) != point {
            return four_points;
        }
        
        let offset = 0.01;
        for i in 0 .. 4 {
            let direction = Vec2::new(
                if i & 1 == 1 { 1. } else { -1. },
                if i & 2 == 1 { 1. } else { -1. }
            );
            four_points[i] = Some(
                ((point - top_left + offset * direction) / block_size).floor().as_ivec2()
            )
        }
        four_points
    }

    fn march_towards_corner(&self, corner:Vec2, start:Vec2, velocity:Vec2) -> Vec2 {
        let distance_to_corner = corner - start;
        //Yeah I divide by 0. No I don't care
        let ticks_to_wall = distance_to_corner/velocity;

        let ticks_to_hit;
        let hit_point = if ticks_to_wall.x > ticks_to_wall.y { 
            //Hitting horizontal wall
            ticks_to_hit = ticks_to_wall.y;
            Vec2::new(start.x + velocity.x*ticks_to_wall.y, corner.y)
        } else if ticks_to_wall.x < ticks_to_wall.y { 
            //Hitting vertical wall
            ticks_to_hit = ticks_to_wall.x;
            Vec2::new(corner.x, start.y + velocity.y * ticks_to_wall.x)
        } else { 
            //Hitting corner
            ticks_to_hit = ticks_to_wall.x;
            corner.clone()
        };
        if ticks_to_hit < 1. {
            println!("Crossing wall!!")
        }
        // println!("Hitting wall in {ticks_to_hit} ticks");

        draw_vec_line(start, corner, 1., ORANGE);
        draw_centered_rect(hit_point, Vec2::splat(10.), YELLOW);

        hit_point
    }

    fn march(&self, world:&SparseDirectedGraph, moving:&Object, max_depth:u32) {
        let cartesian = match self.coord_to_cartesian(moving.position, max_depth + 1)[velocity_to_zorder_direction(moving.velocity)] {
            Some(cartesian) => cartesian.as_vec2(),
            None => {
                println!("Can't march beyond box domain");
                return
            }
        };
        let zorder = cartesian_to_zorder(cartesian.x as u32, cartesian.y as u32, max_depth + 1);
        let depth = match world.read_destination(self.root, &Path2D::from(zorder, max_depth as usize + 1)) {
            Ok((_, depth)) => depth as u32,
            Err(error) => {
                dbg!(error);
                panic!();
            }
        };

        let cur_depth = depth;
        let box_size = self.domain / 2u32.pow(cur_depth) as f32;
        let quadrant = (moving.velocity.signum()+0.5).abs().floor();

        let corner = self.coord_to_cartesian(moving.position, cur_depth)[velocity_to_zorder_direction(moving.velocity)]
        .unwrap()
        .as_vec2() * box_size + box_size*quadrant + self.position - self.domain/2.;

        self.march_towards_corner(corner, moving.position, moving.velocity);

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
            let color = if *index == 0 { BLACK } else if *index == 1 { RED } else if *index == 2 { BLUE } else if *index == 3 { DARKPURPLE } else { GREEN };
            draw_rect(offset, block_domain, color);
            if draw_lines {
                outline_rect(offset, block_domain, 1., WHITE);
            }
        }
    }

    pub fn move_with_collisions(&mut self, moving:&mut Object, hitting:&Object) {
        if moving.velocity.length() != 0. {
            hitting.march(&self.graph, moving, 5);
        } 
        // let drag = 0.99;
        let speed_min = 0.005;
        moving.position += moving.velocity;
        // moving.velocity = moving.velocity * drag;
        if moving.velocity.x.abs() < speed_min {
            moving.velocity.x = 0.;
        }
        if moving.velocity.y.abs() < speed_min {
            moving.velocity.y = 0.;
        }
        // let rot_drag = 0.9;
        // let rot_min = 0.001;
        moving.rotation += moving.angular_velocity;
        moving.rotation %= 2.*PI;
        // moving.angular_velocity *= rot_drag;
        // if moving.angular_velocity.abs() < rot_min {
        //     moving.angular_velocity = 0.
        // }
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
        let cell = cartesian_to_zorder(edit_cell.x as u32, edit_cell.y as u32, depth);
        let path = Path2D::from(cell, depth as usize);

        let child_index = Index( match color {
            BLACK => 0,
            MAROON => 1,
            BLUE => 2,
            DARKPURPLE => 3,
            GREEN => 4,
            _ => 0,
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
fn zorder_to_cartesian(mut zorder:u32, depth:u32) -> IVec2 {
    let (mut x, mut y) = (0, 0);
    for layer in 0 .. depth {
        x |= (zorder & 0b1) << layer;
        zorder >>= 1;
        y |= (zorder & 0b1) << layer;
        zorder >>= 1;
    }
    IVec2::new(x as i32, y as i32)
}

//Figure this out later
fn cartesian_to_zorder(x:u32, y:u32, root_layer:u32) -> usize {
    let mut cell = 0;
    for layer in (0 ..= root_layer).rev() {
        let step = (((y >> layer) & 0b1) << 1 ) | ((x >> layer) & 0b1);
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



