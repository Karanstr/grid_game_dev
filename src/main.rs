use macroquad::prelude::*;
mod sddag;
use sddag::{NodeAddress, SparseDimensionlessDAG, Path};

#[macroquad::main("First Window")]
async fn main() {
    let mut body = DAGBody::new(
        sddag::SparseDimensionlessDAG::new(2), 
        NodeAddress::new(2, 0), 
        Vec2::new(screen_width()/2., screen_height()/2.), 
        20.
    );

    //Window (game) 
    loop {
        
        if is_mouse_button_pressed(MouseButton::Left) {
            body.toggle_cell_with_mouse(Vec2::from(mouse_position()));
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
    dag : SparseDimensionlessDAG,
    root : NodeAddress,
    world_tether : WorldTether
}

impl DAGBody {

    fn new(dag:SparseDimensionlessDAG, root:NodeAddress, position:Vec2, box_size:f32) -> Self {
        Self {
            dag,
            root,
            world_tether : WorldTether::new(position, Vec2::new(box_size, box_size))
        }
    }

    //This is ugly clean it up. I dislike recomputing edit_cell.
    fn toggle_cell_with_mouse(&mut self, mouse_pos:Vec2) {
        let rel_mouse_pos = mouse_pos - self.world_tether.position;
        let unrounded_cell = rel_mouse_pos / self.world_tether.size;
        let mut edit_cell = IVec2::new(
            round_away_0_pref_pos(unrounded_cell.x),
            round_away_0_pref_pos(unrounded_cell.y)
        );
        let blocks_on_side = 2i32.pow(self.root.layer as u32);
        if edit_cell.abs().max_element() > blocks_on_side { return }
        edit_cell += blocks_on_side;
        if edit_cell.x > blocks_on_side { edit_cell.x -= 1 }
        if edit_cell.y > blocks_on_side { edit_cell.y -= 1 }


        let cell = xy_to_cell(edit_cell.x as u32, edit_cell.y as u32, self.root.layer);
        let path = Path::from(cell, self.root.layer + 1, 2);
        let cur_node_val = self.dag.read_node_child(&self.root, 0, &path);
        let new_val = if cur_node_val == 0 { 1 } else { 0 } as usize;
        self.dag.set_node_child(&mut self.root, 0, &path, new_val);
    }

    fn render(&self) {
        let blocks_on_side = 2u32.pow(self.root.layer as u32);
        let cell_count = (blocks_on_side*2).pow(2);
        for cell in 0 .. cell_count {
            let cur_cell_path = Path::from(cell, self.root.layer + 1, 2);
            let cur_cell = self.dag.read_node_child(&self.root, 0, &cur_cell_path);
            let color = if cur_cell == 0 { RED } else { BLUE };
            let xy = cell_to_xy(cell, self.root.layer);
            let half_size = Vec2::splat(blocks_on_side as f32) * self.world_tether.size;
            let offset = Vec2::new(xy.x as f32, xy.y as f32) * self.world_tether.size - half_size;
            draw_rectangle(self.world_tether.position.x + offset.x, self.world_tether.position.y + offset.y, self.world_tether.size.x, self.world_tether.size.y, color);
        }

        draw_rectangle(self.world_tether.position.x - 5., self.world_tether.position.y - 5., 10., 10., GREEN);
    }

    // //Not optimal without recursing the entire tree, our current first order search (2 layers deep is meh). Consider how much I care
    // fn minimize_space_used(&mut self) {
    //     let blocks_shifted = self.dag.shrink_root_to_fit(&mut self.root);
    //     self.world_tether.position.x += blocks_shifted as f32 * self.world_tether.size.x;
    // }

    // fn double_capacity(&mut self, side_preserve:usize) {
    //     let blocks_shifted = self.dag.raise_root_by_one(&mut self.root, side_preserve);
    //     self.world_tether.position.x += blocks_shifted as f32 * self.world_tether.size.x; 
    // }

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

pub fn cell_to_xy(mut cell:u32, root_layer:usize) -> UVec2 {
    let mut xy = UVec2::new(0, 0);
    for layer in 0 ..= root_layer {
        xy.x |= (cell & 0b1) << layer;
        cell >>= 1;
        xy.y |= (cell & 0b1) << layer;
        cell >>= 1;
    }
    xy
}

pub fn xy_to_cell(x:u32, y:u32, root_layer:usize) -> u32 {
    let mut cell = 0;
    for layer in (0 ..= root_layer).rev() {
        let step = (((y >> layer) & 0b1) << 1 ) | ((x >> layer) & 0b1);
        cell = (cell << 2) | step;
    }
    cell
}