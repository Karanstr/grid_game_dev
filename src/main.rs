use std::f32::consts::PI;
use macroquad::{
    prelude::*,
    rand::gen_range,
};
use vec_friendly_drawing::*;

mod graph;
use graph::{SparseDirectedGraph, Path2D, Index, NodePointer};

#[macroquad::main("Window")]
async fn main() {
    let size = Vec2::new(1100., 1100.);
    request_new_screen_size(size.x, size.y);
    let mut world_graph = SparseDirectedGraph::new();
    let mut world = Object {
        root : world_graph.empty_root(),
        position : Vec2::new(size.x/2., size.y/2.),
        domain : Vec2::new(size.x, size.y),
    };
    let mut player = Player::new(WHITE, size/2.);
    let speed = 0.15;
    let step = 0.1;
    let mut operation_depth = 1;
    let mut cur_color = MAROON;

    //Keeps window alive, window closes when main terminates (Figure out how that works)
    loop {

        if is_key_pressed(KeyCode::P) {
            world_graph.profile();
        } else if is_key_pressed(KeyCode::C) {
            world.root = world_graph.clear_root(world.root);
        } else if is_key_pressed(KeyCode::R) {
            let depth = 3;
            let steps = 20;
            world.root = world_graph.clear_root(world.root);
            world.color_war(&mut world_graph, depth, steps);
            world_graph.profile();
        } else if is_key_pressed(KeyCode::V) {
            cur_color = match cur_color {
                BLACK => MAROON,
                MAROON => BLUE,
                BLUE => DARKPURPLE,
                DARKPURPLE => GREEN,
                GREEN => BLACK,
                _ => BLACK
            }
        }

        if is_key_pressed(KeyCode::Key1) {
            operation_depth = 1;
        } else if is_key_pressed(KeyCode::Key2) {
            operation_depth = 2;
        } else if is_key_pressed(KeyCode::Key3) {
            operation_depth = 3;
        } else if is_key_pressed(KeyCode::Key4) {
            operation_depth = 4;
        } else if is_key_pressed(KeyCode::Key5) {
            operation_depth = 5;
        } else if is_key_pressed(KeyCode::Key6) {
            operation_depth = 6;
        } else if is_key_pressed(KeyCode::Key7) {
            operation_depth = 7;
        } else if is_key_pressed(KeyCode::Key8) {
            operation_depth = 8;
        }

        
        if is_mouse_button_down(MouseButton::Left) {
            world.set_cell_with_mouse(&mut world_graph, Vec2::from(mouse_position()), operation_depth, cur_color);
        }
       
        world.render(&world_graph, true);
        player.apply_acceleration(speed, step);
        if player.velocity.length() != 0. {
            world.march(&world_graph, &player.position, &player.velocity, operation_depth);
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

    fn apply_acceleration(&mut self, linear:f32, rotational:f32) {
        if is_key_down(KeyCode::A) {
            self.rotation -= rotational;
        }
        if is_key_down(KeyCode::D) {
            self.rotation += rotational;
        }
        self.rotation %= 2.*PI;
        if is_key_down(KeyCode::W) {
            self.velocity += Vec2::new(linear * self.rotation.cos(),linear * self.rotation.sin());
        }
        if is_key_down(KeyCode::S) {
            self.velocity += -1. * Vec2::new(linear * self.rotation.cos(),linear * self.rotation.sin());
        }
    }

    fn vel_to_pos(&mut self) {
        let drag = 0.99;
        let speed_min = 0.01;
        self.position += self.velocity;
        self.velocity = self.velocity * drag;
        if self.velocity.x.abs() < speed_min {
            self.velocity.x = 0.;
        }
        if self.velocity.y.abs() < speed_min {
            self.velocity.y = 0.;
        }
    }

}


struct Object {
    root : NodePointer,
    position : Vec2,
    domain : Vec2,
}

impl Object {

    fn render(&self, graph:&SparseDirectedGraph, draw_lines:bool) {
        let filled_blocks = graph.dfs_leaves(self.root);
        for (zorder, depth, index) in filled_blocks {
            let block_domain = self.domain / 2u32.pow(depth) as f32;
            let cartesian_cell = zorder_to_cartesian(zorder, depth);
            let offset = Vec2::new(cartesian_cell.x as f32, cartesian_cell.y as f32) * block_domain + self.position - self.domain/2.;
            let color = if *index == 0 { BLACK } else if *index == 1 { RED } else if *index == 2 { BLUE } else if *index == 3 { DARKPURPLE } else { GREEN };
            draw_rect(offset, block_domain, color);
            if draw_lines {
                outline_rect(offset, block_domain, 1., WHITE);
            }
        }
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

    fn set_cell_with_mouse(&mut self, graph:&mut SparseDirectedGraph, mouse_pos:Vec2, depth:u32, color:Color) {
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

        let child_index = Index( match color {
            BLACK => 0,
            MAROON => 1,
            BLUE => 2,
            DARKPURPLE => 3,
            GREEN => 4,
            _ => 0,
        } );

        let new_child = NodePointer::new(child_index, 0b0000);

        if let Ok(root) = graph.set_node_child(self.root, &path, new_child) {
            self.root = root
        };
    }

    fn fill_walk(&mut self, graph:&mut SparseDirectedGraph, start:IVec2, depth:u32, steps:usize, data:usize) {
        let max = IVec2::splat(2i32.pow(depth) - 1);
        // let mut current = IVec2::new(gen_range(0, max.x+1) as i32, gen_range(0, max.y+1) as i32);
        let mut current = start;
        for _ in 0 .. steps as u32 {
            let path = Path2D::from(cartesian_to_zorder(current.x as u32, current.y as u32, depth), depth as usize);
            current.x += gen_range(0, 3) as i32 - 1;
            current.y += gen_range(0, 3) as i32 - 1;
            current = current.clamp(IVec2::ZERO, max);
            if let Ok(root) = graph.set_node_child(self.root, &path, NodePointer::new(Index(data), 0b0000)) {
                self.root = root
            };
        }
    }

    fn color_war(&mut self, graph:&mut SparseDirectedGraph, depth:u32, steps:usize) {
        let max = 2i32.pow(depth) - 1;
        let mut positions = [IVec2::ZERO, IVec2::new(0, max), IVec2::new(max, max), IVec2::new(max, 0)];
        for _ in 0 .. steps as u32 {
            for i in 0 .. 4 {
                let path = Path2D::from(cartesian_to_zorder(positions[i].x as u32, positions[i].y as u32, depth), depth as usize);
                positions[i].x += gen_range(0, 3) as i32 - 1;
                positions[i].y += gen_range(0, 3) as i32 - 1;
                positions[i] = positions[i].clamp(IVec2::ZERO, IVec2::splat(max));
                if let Ok(root) = graph.set_node_child(self.root, &path, NodePointer::new(Index(i + 1), 0b0000)) {
                    self.root = root
                };
            }
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

    fn march(&self, world:&SparseDirectedGraph, start:&Vec2, velocity:&Vec2, max_depth:u32) {
        let cartesian = match self.coord_to_cartesian(*start, max_depth + 1)[velocity_to_zorder_direction(velocity)] {
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
        let quadrant = (velocity.signum()+0.5).abs().floor();

        let corner = self.coord_to_cartesian(*start, cur_depth)[velocity_to_zorder_direction(velocity)]
        .unwrap()
        .as_vec2() * box_size + box_size*quadrant + self.position - self.domain/2.;

        self.march_towards_corner(corner, *start, *velocity);

    }


}


fn velocity_to_zorder_direction(velocity:&Vec2) -> usize {
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

