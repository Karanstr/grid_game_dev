use macroquad::prelude::*;
use graph::{SparseDirectedGraph, Index, Path};
mod graph;
mod garbagetracker;


// const BOXSIZE:Vec2 = Vec2::splat(40.);


#[macroquad::main("First Window")]
async fn main() {

    let mut sparse_graph: SparseDirectedGraph = SparseDirectedGraph::new();
    let mut root = sparse_graph.get_empty_root();

    let new_path = Path::from(0b11, 1, 2);
    root = sparse_graph.set_node_child(root, &new_path, Index(1));
    let new_path = Path::from(0b00, 1, 2);
    root = sparse_graph.set_node_child(root, &new_path, root);
    root = sparse_graph.set_node_child(root, &new_path, Index(1));
    dbg!(*root);

    //Keeps window alive
    loop {
       

        next_frame().await
    }

}

//Update all this object stuff to work with the new (gamer) graph layout
// struct Object {
//     root : NodeAddress,
//     position : Vec2,
// }

// impl Object {

//     fn render(&self, dag:&SparseDimensionlessDAG) {
//         let blocks_on_side = 2usize.pow(self.root.layer as u32);
//         let cell_count = (blocks_on_side*2).pow(2);
//         //Probably a better way to do this, but I don't care. Rendering is not currently the bottleneck, and optimizing early will just waste time.
//         for cell in 0 .. cell_count {
//             let path = Path::from(cell, self.root.layer + 1, 2);
//             let color = match dag.read_end_of_path(&self.root, &path) {
//                 Some(val) => if val == 0 { RED} else { BLUE },
//                 None => RED
//             };
//             let cartesian_cell = zorder_to_cartesian(cell, self.root.layer) - blocks_on_side as i32;
//             let offset = Vec2::new(cartesian_cell.x as f32, cartesian_cell.y as f32) * BOXSIZE;
    
//             draw_rectangle(
//                 self.position.x + offset.x,
//                 self.position.y + offset.y, 
//                 BOXSIZE.x, 
//                 BOXSIZE.y, 
//                 color);
//         }
//         //Draw center of box
//         draw_rectangle(self.position.x - 5., self.position.y - 5., 10., 10., GREEN);
//     }

//     fn move_with_wasd(&mut self, speed:f32) {
//         if is_key_down(KeyCode::A) {
//             self.position.x -= speed;
//         }
//         if is_key_down(KeyCode::D) {
//             self.position.x += speed;
//         }
//         if is_key_down(KeyCode::W) {
//             self.position.y -= speed;
//         }
//         if is_key_down(KeyCode::S) {
//             self.position.y += speed;
//         }
//     }

//     fn toggle_cell_with_mouse(&mut self, sddag:&mut SparseDimensionlessDAG, mouse_pos:Vec2) {
//         let rel_mouse_pos = mouse_pos - self.position;
//         let unrounded_cell = rel_mouse_pos / BOXSIZE;
//         let mut edit_cell = IVec2::new(
//             round_away_0_pref_pos(unrounded_cell.x),
//             round_away_0_pref_pos(unrounded_cell.y)
//         );
//         let blocks_on_side = 2i32.pow(self.root.layer as u32);
//         if edit_cell.abs().max_element() > blocks_on_side { return }
//         edit_cell += blocks_on_side;
//         if edit_cell.x > blocks_on_side { edit_cell.x -= 1 }
//         if edit_cell.y > blocks_on_side { edit_cell.y -= 1 }

//         let cell = cartesian_to_zorder(edit_cell.x as usize, edit_cell.y as usize, self.root.layer);
//         let path = Path::from(cell, self.root.layer + 1, 2);
//         let cur_val = match sddag.read_end_of_path(&self.root, &path) {
//             Some(value) => value,
//             None => 0
//         };
//         let new_val = if cur_val == 0 { 1 } else { 0 };
//         sddag.set_node_child(&mut self.root, &path, new_val, true);
//         dbg!(sddag.df_to_bin_grid(&self.root));
//     }

// }


// //Figure out where to put these
// fn round_away_0_pref_pos(number:f32) -> i32 {
//     if number < 0. {
//         number.floor() as i32
//     } else if number > 0. {
//         number.ceil() as i32
//     }
//     else {
//         //We don't want to return 0 when we're trying to avoid 0
//         //And the name of the function is prefer_position, so..
//         1 
//     }
// }

// //Will overflow if our z-order goes 32 layers deep. So.. don't do that
// pub fn zorder_to_cartesian(cell:usize, root_layer:usize) -> IVec2 {
//     let mut u32_cell = cell as i32;
//     let (mut x, mut y) = (0, 0);
//     for layer in 0 ..= root_layer {
//         x |= (u32_cell & 0b1) << layer;
//         u32_cell >>= 1;
//         y |= (u32_cell & 0b1) << layer;
//         u32_cell >>= 1;
//     }
//     IVec2::new(x, y)
// }

// pub fn cartesian_to_zorder(x:usize, y:usize, root_layer:usize) -> usize {
//     let mut cell = 0;
//     for layer in (0 ..= root_layer).rev() {
//         let step = (((y >> layer) & 0b1) << 1 ) | ((x >> layer) & 0b1);
//         cell = (cell << 2) | step;
//     }
//     cell
// }

