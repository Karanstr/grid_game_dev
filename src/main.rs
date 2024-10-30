use core::panic;

use macroquad::prelude::*;
use graph::{DirectedGraph, Index, Path};
mod graph;
mod garbagetracker;


const BOXSIZE:Vec2 = Vec2::splat(50.);
const DESCENT_LIMIT:u32 = 3;

#[macroquad::main("Window")]
async fn main() {

    let mut world_graph = DirectedGraph::new();

    let mut player = Object {
        root : world_graph.get_empty_root(),
        position : Vec2::new(screen_width()/2., screen_height()/2.)
    };

    //Keeps window alive
    loop {
       
        if is_mouse_button_pressed(MouseButton::Left) {
            player.toggle_cell_with_mouse(&mut world_graph, Vec2::from(mouse_position()));
        }
        if is_key_pressed(KeyCode::P) {
            println!("Paused.");
        }

        player.move_with_wasd(5.);

        player.render(&world_graph);

        next_frame().await
    }

}

struct Object {
    root : Index,
    position : Vec2,
}

impl Object {

    fn render(&self, dag:&DirectedGraph) {
        let blocks_per_face = 2usize.pow(DESCENT_LIMIT);
        let cell_count: usize = blocks_per_face.pow(2); //Squared

        for cell in 0 .. cell_count {
            let path = Path::from(cell, DESCENT_LIMIT as usize, 2);
            let color = match dag.read_destination(self.root, &path) {
                Ok(val) => if *val == 0 { RED } else { BLUE },
                Err( error ) => {
                    dbg!(error);
                    RED
                }
            };
            let cartesian_cell = zorder_to_cartesian(cell, DESCENT_LIMIT as u32) - blocks_per_face as i32/2;
            let offset = Vec2::new(cartesian_cell.x as f32, cartesian_cell.y as f32) * BOXSIZE;
    
            draw_rectangle(
                self.position.x + offset.x,
                self.position.y + offset.y, 
                BOXSIZE.x, 
                BOXSIZE.y, 
                color);
        }
        //Draw center of box
        draw_rectangle(self.position.x - 5., self.position.y - 5., 10., 10., GREEN);
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

    fn toggle_cell_with_mouse(&mut self, graph:&mut DirectedGraph, mouse_pos:Vec2) {
        let rel_mouse_pos = mouse_pos - self.position;
        let unrounded_cell = rel_mouse_pos / BOXSIZE;
        let mut edit_cell = IVec2::new(
            round_away_0_pref_pos(unrounded_cell.x),
            round_away_0_pref_pos(unrounded_cell.y)
        );
        let blocks_on_half = 2i32.pow(DESCENT_LIMIT - 1);
        if edit_cell.abs().max_element() > blocks_on_half { return }
        edit_cell += blocks_on_half;
        if edit_cell.x > blocks_on_half { edit_cell.x -= 1 }
        if edit_cell.y > blocks_on_half { edit_cell.y -= 1 }
        let cell = cartesian_to_zorder(edit_cell.x as usize, edit_cell.y as usize, DESCENT_LIMIT);
        let path = Path::from(cell, DESCENT_LIMIT as usize, 2);
        let cur_val = match graph.read_destination(self.root, &path) {
            Ok(value) => *value,
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

//Will overflow if our z-order goes 32 layers deep. So.. don't do that
pub fn zorder_to_cartesian(cell:usize, root_layer:u32) -> IVec2 {
    let mut u32_cell = cell as i32;
    let (mut x, mut y) = (0, 0);
    for layer in 0 .. root_layer {
        x |= (u32_cell & 0b1) << layer;
        u32_cell >>= 1;
        y |= (u32_cell & 0b1) << layer;
        u32_cell >>= 1;
    }
    IVec2::new(x, y)
}

pub fn cartesian_to_zorder(x:usize, y:usize, root_layer:u32) -> usize {
    let mut cell = 0;
    for layer in (0 ..= root_layer).rev() {
        let step = (((y >> layer) & 0b1) << 1 ) | ((x >> layer) & 0b1);
        cell = (cell << 2) | step;
    }
    cell
}

