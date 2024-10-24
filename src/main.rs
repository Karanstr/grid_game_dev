use macroquad::prelude::*;
mod sddag;
use sddag::{NodeAddress, Path, SparseDimensionlessDAG};

const BOXSIZE:Vec2 = Vec2::splat(20.);


struct Object {
    root : NodeAddress,
    position : Vec2,
}


impl Object {

    fn render(&self, dag:&SparseDimensionlessDAG) {
        let blocks_on_side = 2u32.pow(self.root.layer as u32);
        let cell_count = (blocks_on_side*2).pow(2);
        //Probably a better way to do this, but I don't care. Rendering is not currently the bottleneck, and optimizing early will just waste time.
        for cell in 0 .. cell_count {
            let cur_cell = dag.read_node_child(
                &self.root, 
                0, 
                &Path::from(cell, self.root.layer + 1, 2)
            );
            let color = if cur_cell == 0 { RED } else { BLUE };

            let cartesian_cell = zorder_to_cartesian(cell, self.root.layer);
            let offset = (Vec2::new(cartesian_cell.x as f32, cartesian_cell.y as f32) - Vec2::splat(blocks_on_side as f32)) * BOXSIZE;
    
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

    fn toggle_cell_with_mouse(&mut self, sddag:&mut SparseDimensionlessDAG, mouse_pos:Vec2) {
        let rel_mouse_pos = mouse_pos - self.position;
        let unrounded_cell = rel_mouse_pos / BOXSIZE;
        let mut edit_cell = IVec2::new(
            round_away_0_pref_pos(unrounded_cell.x),
            round_away_0_pref_pos(unrounded_cell.y)
        );
        let blocks_on_side = 2i32.pow(self.root.layer as u32);
        if edit_cell.abs().max_element() > blocks_on_side { return }
        edit_cell += blocks_on_side;
        if edit_cell.x > blocks_on_side { edit_cell.x -= 1 }
        if edit_cell.y > blocks_on_side { edit_cell.y -= 1 }

        let cell = cartesian_to_zorder(edit_cell.x as u32, edit_cell.y as u32, self.root.layer);
        let path = Path::from(cell, self.root.layer + 1, 2);
        let cur_node_val = sddag.read_node_child(&self.root, 0, &path);
        let new_val = if cur_node_val == 0 { 1 } else { 0 } as usize;
        sddag.set_node_child(&mut self.root, 0, &path, new_val);
    }


}


#[macroquad::main("First Window")]
async fn main() {

    let mut sddag = SparseDimensionlessDAG::new(2);

    let mut player = Object {
        root : NodeAddress::new(2, 0),
        position : Vec2::new(screen_width()/2., screen_height()/2.),
    };

    //Keeps window alive
    loop {

        if is_mouse_button_pressed(MouseButton::Left) {
            player.toggle_cell_with_mouse(&mut sddag, Vec2::from(mouse_position()));
            //dbg!(sddag.df_to_bin_grid(&player.root));
        } 

        player.move_with_wasd(5.);

        player.render(&sddag);

        next_frame().await
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

pub fn zorder_to_cartesian(mut cell:u32, root_layer:usize) -> UVec2 {
    let mut xy = UVec2::new(0, 0);
    for layer in 0 ..= root_layer {
        xy.x |= (cell & 0b1) << layer;
        cell >>= 1;
        xy.y |= (cell & 0b1) << layer;
        cell >>= 1;
    }
    xy
}

pub fn cartesian_to_zorder(x:u32, y:u32, root_layer:usize) -> u32 {
    let mut cell = 0;
    for layer in (0 ..= root_layer).rev() {
        let step = (((y >> layer) & 0b1) << 1 ) | ((x >> layer) & 0b1);
        cell = (cell << 2) | step;
    }
    cell
}

