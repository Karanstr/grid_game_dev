use core::panic;
use std::{f32::consts::PI, os::linux::raw::stat};

use macroquad::prelude::*;
use vec_friendly_drawing::*;
use graph::{SparsePixelDirectedGraph, Index, Path2D};
mod graph;
mod fake_heap;

#[macroquad::main("Window")]
async fn main() {
    let size = Vec2::new(300., 300.);
    request_new_screen_size(size.x, size.y);
    let mut world_graph = SparsePixelDirectedGraph::new();
    let mut player = Object {
        root : world_graph.empty_root(),
        position : Vec2::new(size.x/2., size.y/2.),
        domain : Vec2::new(size.x, size.y),
    };

    let step = 0.03;
    let mut rotation = step;

    //Keeps window alive
    loop {
       
        if is_mouse_button_pressed(MouseButton::Left) {
            player.toggle_cell_with_mouse(&mut world_graph, Vec2::from(mouse_position()));
        }
        // if is_key_pressed(KeyCode::P) {
        //     println!("Paused.");
        // }

        if is_key_down(KeyCode::W) {
            if rotation > step {
                rotation -= step;
            }
        }
        if is_key_down(KeyCode::S) {
            if rotation < PI/2. - step {
                rotation += step;
            }
        }

        //player.move_with_wasd(5.);
        
        player.render(&world_graph);

        player.march_down_and_right(&world_graph, Vec2::from(mouse_position()), Vec2::from_angle(rotation));

        next_frame().await
    }

}

struct Object {
    root : Index,
    position : Vec2,
    domain : Vec2,
}


impl Object {

    fn render(&self, graph:&SparsePixelDirectedGraph) {
        let filled_blocks = graph.dfs_leaves(self.root);
        for (zorder, depth, index) in filled_blocks {
            let block_domain = self.domain / 2u32.pow(depth) as f32;
            let cartesian_cell = zorder_to_cartesian(zorder, depth);
            let offset = Vec2::new(cartesian_cell.x as f32, cartesian_cell.y as f32) * block_domain + self.position - self.domain/2.;
            let color = if *index == 0 { RED } else { BLUE };
            draw_rect(offset, block_domain, color);
            outline_rect(offset, block_domain, 2., WHITE);
        }
        draw_centered_rect(self.position, Vec2::splat(10.), GREEN);
    }

    fn move_with_wasd(&mut self, speed:f32) {
        if is_key_down(KeyCode::A) {
            self.position.x -= speed;
        }
        if is_key_down(KeyCode::D) {
            self.position.x += speed;
        }
        if is_key_down(KeyCode::W) {
            self.position.y -= speed;
        }
        if is_key_down(KeyCode::S) {
            self.position.y += speed;
        }
    }

    fn toggle_cell_with_mouse(&mut self, graph:&mut SparsePixelDirectedGraph, mouse_pos:Vec2) {
        let depth = 2;
        let block_size = self.domain / 2u32.pow(depth) as f32;

        let rel_mouse_pos = mouse_pos - self.position;
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
        let cur_val = match graph.read_destination(self.root, &path) {
            Ok(value) => *value.index,
            Err( error ) => {
                dbg!(error);
                0   
            }
        };
        let new_val = match cur_val {
            1 => Index(0),
            0 => Index(1),
            _ => {
                dbg!(cur_val);
                Index(0)
            }
        };
        self.root = match graph.set_node_child(self.root, &path, new_val) {
            Ok(index) => index,
            Err( error ) => {
                dbg!( error );
                //If something goes really wrong here the object isn't recoverable.
                //Root has to remain pointing to a valid address, otherwise everything spirals
                panic!();
            }
        };
    }

    fn coord_to_cartesian(&self, point:Vec2, depth:u32) -> IVec2 {
        let blocks = Vec2::splat(2f32.powf(depth as f32));
        if point.x < self.position.x - self.domain.x/2. || point.y < self.position.y - self.domain.y/2. {
            println!("Bad");
        }
        (point / (self.domain / blocks)).floor().as_ivec2()
    }

    //Assumes the corner is down and to the right
    //Assumes the velocity isn't vertical only (inf slope)
    fn march_towards_corner(&self, corner:Vec2, start:Vec2, velocity:Vec2) -> Vec2 {
        let distance_to_corner = corner - start;
        let slope = velocity.y / velocity.x;
        let slope_to_corner = distance_to_corner.y / distance_to_corner.x;

        let hit_point = if slope_to_corner < slope {
            println!("Hitting horizontal wall");
            Vec2::new(start.x + distance_to_corner.y/slope, corner.y)
        } else if slope_to_corner > slope {
            println!("Hitting vertical wall");
            Vec2::new(corner.x, start.y + distance_to_corner.x * slope)
        } else {
            println!("Hitting corner");
            corner.clone()
        };

        draw_vec_line(start, corner, 1., ORANGE);
        draw_vec_line(start, hit_point, 1., GREEN);

        draw_centered_rect(hit_point, Vec2::splat(10.), YELLOW);

        hit_point

    }

    fn march_down_and_right(&self, world:&SparsePixelDirectedGraph, start:Vec2, velocity:Vec2) {
        
        let max_depth = 5;
        let cartesian = self.coord_to_cartesian(start, max_depth).as_vec2();
        let zorder = cartesian_to_zorder(cartesian.x as u32, cartesian.y as u32, max_depth);


        let data = match world.read_destination(self.root, &Path2D::from(zorder, max_depth as usize)) {
            Ok(data) => data,
            Err(error) => {
                dbg!(error);
                panic!();
            }
        };

        let cur_depth = data.depth as u32 - 1;


        let box_size = self.domain / 2u32.pow(cur_depth) as f32;
        let bottom_right = self.coord_to_cartesian(start, cur_depth).as_vec2() * box_size + box_size;
        
        self.march_towards_corner(bottom_right, start, velocity);

    }


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

