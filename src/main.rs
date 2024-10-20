use macroquad::prelude::*;
use sddag::{NodeAddress, SparseDAG1D};
use std::i32::MAX;
mod sddag;

#[macroquad::main("First Window")]
async fn main() {

    //Set-up
    let dag  = sddag::SparseDAG1D::new(0);
    let root = NodeAddress::new(0, 0);
    let mut body = DAGBody::new(dag, root, Vec2::new(screen_width()/2., screen_height()/2.), 20.);
    
    let mut last_edit_cell:i32 = MAX;

    //Window (game) loop
    loop {
        
        //Stupid code repeat, maybe abstract into toggle_cell_drag()?
        //Makes sure while the mouse is held you won't change a cell you just changed.
        let rel_mouse_pos = Vec2::from(mouse_position()) - body.world_tether.position;
        let edit_cell: i32 = round_away_0_pref_pos(rel_mouse_pos.x / body.world_tether.size.x);
        if is_mouse_button_down(MouseButton::Left) && edit_cell != last_edit_cell {
            body.toggle_cell_with_mouse(Vec2::from(mouse_position()));
            last_edit_cell = edit_cell;
        } else if is_mouse_button_released(MouseButton::Left) {
            last_edit_cell = MAX
        }

        if is_key_pressed(KeyCode::F) {
            body.dag.raise_root_by_one(&mut body.root, 0);
        }
        if is_key_pressed(KeyCode::G) {
            body.dag.lower_root_by_one(&mut body.root, 0);
        }
        if is_key_pressed(KeyCode::O) {
            body.dag.compress_root_once(&mut body.root);
        }


        body.world_tether.move_with_wasd(5.);




        body.render();

        next_frame().await
    }
}


struct WorldTether {
    position : Vec2,
    velocity : Vec2,
    size : Vec2,
}

#[allow(dead_code)]
impl WorldTether {

    fn new(pos:Vec2, dimensions:Vec2) -> Self {
        Self {
            position : pos,
            velocity : Vec2::new(0., 0.),
            size : dimensions
        }
    }

    fn update_position(&mut self) {
        self.position += self.velocity;
        self.velocity.x = 0.;
        self.velocity.y = 0.;
    }

    fn velocity_as_wasd(&mut self, speed:f32) {
        if is_key_down(KeyCode::A) {
            self.velocity.x -= speed;
        }
        if is_key_down(KeyCode::D) {
            self.velocity.x += speed;
        }
        if is_key_down(KeyCode::W) {
            self.velocity.y -= speed;
        }
        if is_key_down(KeyCode::S) {
            self.velocity.y += speed;
        }
    }

    fn move_with_wasd(&mut self, speed:f32) {
        self.velocity_as_wasd(speed);
        self.update_position();
    }

}

struct DAGBody {
    dag : SparseDAG1D,
    root : NodeAddress,
    world_tether : WorldTether
}

impl DAGBody {

    fn new(dag:SparseDAG1D, root:NodeAddress, position:Vec2, box_size:f32) -> Self {
        Self {
            dag,
            root,
            world_tether : WorldTether::new(position, Vec2::new(box_size, box_size))
        }
    }

    fn toggle_cell_with_mouse(&mut self, mouse_pos:Vec2) {
        let rel_mouse_pos = mouse_pos - self.world_tether.position;
        let edit_cell: i32 = round_away_0_pref_pos(rel_mouse_pos.x / self.world_tether.size.x);
        let blocks_on_side = 2i32.pow(self.root.layer as u32);
        //If mouse is within bounds. Eventually we add an else to expand the DAG if some parameter is true
        if edit_cell.abs() <= blocks_on_side {
            let path = {
                if edit_cell < 0 {
                    edit_cell + blocks_on_side
                } else {
                    edit_cell + blocks_on_side - 1
                }
            } as u32;
            let cur_node_val = self.dag.read_node_child(&self.root, 0, path);
            let new_val = if cur_node_val == 0 { 1 } else { 0 } as usize;
            self.dag.set_node_child(&mut self.root, 0, path, new_val);
        }
    }

    fn render(&self) {
        let tree_bin = self.dag.df_to_binary(&self.root);
        let max_cell = 2i32.pow((self.root.layer + 1) as u32);
        for cell in 0 .. max_cell {
            let cur_cell = (tree_bin >> cell) & 0b1;
            let color = if cur_cell == 0 { RED } else { BLUE };
            let cell_offset = (cell - max_cell/2) as f32 * self.world_tether.size.x;
            draw_rectangle(self.world_tether.position.x + cell_offset, self.world_tether.position.y, self.world_tether.size.x, self.world_tether.size.y, color);
        }
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
        //And the name of the function is prefer_position, so..
        1 
    }
}

