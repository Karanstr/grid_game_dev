use core::panic;
use std::f32::consts::PI;

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
    let mut world = Object {
        root : world_graph.empty_root(),
        position : Vec2::new(size.x/2., size.y/2.),
        domain : Vec2::new(size.x, size.y),
    };

    let mut player = Player::new(PURPLE, size/2.);

    let speed = 2.;
    let step = 0.1;

    
    //Keeps window alive, window closes when main terminates (Figure out how that works)
    loop {

        if is_mouse_button_pressed(MouseButton::Left) {
            world.toggle_cell_with_mouse(&mut world_graph, Vec2::from(mouse_position()));
        }
       

        world.render(&world_graph);
        player.update_orientation_and_velocity_wasd(speed, step);
        
        if player.velocity.length() != 0. {
            world.march(&world_graph, &player.position, &player.velocity);
        }
        player.vel_to_pos();
        player.render();
        next_frame().await
    }

}

struct Player {
    color : Color,
    position : Vec2,
    velocity : Vec2,
    rotation : f32,
}

impl Player {

    fn new(color:Color, position:Vec2) -> Self {
        Player {
            color,
            position,
            velocity : Vec2::ZERO,
            rotation : 0.,
        }
    }

    fn render(&self) {
        draw_centered_rect(self.position, Vec2::splat(10.), self.color);
        draw_vec_line(self.position, self.position + Vec2::new(10. * self.rotation.cos(),10. * self.rotation.sin()), 1., YELLOW);
    }

    fn update_orientation_and_velocity_wasd(&mut self, linear:f32, rotational:f32) {
        if is_key_down(KeyCode::A) {
            self.rotation -= rotational;
        }
        if is_key_down(KeyCode::D) {
            self.rotation += rotational;
        }
        self.rotation %= 2.*PI;
        if is_key_down(KeyCode::W) {
            self.velocity = Vec2::new(linear * self.rotation.cos(),linear * self.rotation.sin());
        }
        if is_key_down(KeyCode::S) {
            self.velocity = -1. * Vec2::new(linear * self.rotation.cos(),linear * self.rotation.sin());
        }
    }

    fn vel_to_pos(&mut self) {
        self.position += self.velocity;
        self.velocity = Vec2::ZERO;
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

    fn _move_with_wasd(&mut self, speed:f32) {
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

    fn coord_to_cartesian(&self, point:Vec2, depth:u32) -> Option<IVec2> {
        let offset = self.domain/2.;
        let blocks = Vec2::splat(2f32.powf(depth as f32));
        if point.x <= self.position.x - offset.x || point.y <= self.position.y - offset.y
        || point.x >= self.position.x + offset.x || point.y >= self.position.y + offset.y {
            return None
        }
        Some(((point - (self.position - offset)) / (self.domain / blocks)).floor().as_ivec2())
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

        draw_vec_line(start, corner, 1., GREEN);
        draw_centered_rect(hit_point, Vec2::splat(10.), YELLOW);

        hit_point
    }

    fn march(&self, world:&SparsePixelDirectedGraph, start:&Vec2, velocity:&Vec2) {
        let max_depth = 5;
        let cartesian = match self.coord_to_cartesian(*start, max_depth) {
            Some(cartesian) => cartesian.as_vec2(),
            None => {
                println!("Can't march beyond box domain");
                return
            }
        };
        let zorder = cartesian_to_zorder(cartesian.x as u32, cartesian.y as u32, max_depth);
        let data = match world.read_destination(self.root, &Path2D::from(zorder, max_depth as usize)) {
            Ok(data) => data,
            Err(error) => {
                dbg!(error);
                panic!();
            }
        };

        let cur_depth = data.depth as u32;
        let box_size = self.domain / 2u32.pow(cur_depth) as f32;
        let quadrant = (velocity.signum()+0.5).abs().floor();

        let corner = self.coord_to_cartesian(*start, cur_depth)
            .unwrap()
            .as_vec2() * box_size + box_size*quadrant + self.position - self.domain/2.;

        self.march_towards_corner(corner, *start, *velocity);

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

