use glam::Vec2;
use crate::engine::grid::*;


// Extract corners into our collision detection system (once we write it)
#[derive(Debug, Clone, derive_new::new)]
pub struct Corners {
    pub points : [Vec2; 4],
    pub index : Index,
}

pub mod corner_handling {
    use glam::UVec2;
    use super::*;

    //The top left corner of the root is (0, 0)
    fn cell_corners(cell: UVec2, height: u32, min_cell_length: Vec2) -> [Vec2; 4] {
        let cell_size = cell_length(height, min_cell_length);
        let top_left_corner = cell.as_vec2() * cell_size;
        [
            top_left_corner,
            top_left_corner.with_x(top_left_corner.x + cell_size.x),
            top_left_corner.with_y(top_left_corner.y + cell_size.y),
            top_left_corner + cell_size,
        ]
    }

    // pub fn tree_corners(head: ExternalPointer, min_cell_length:Vec2) -> Vec<Corners> {
    //     let leaves = dfs_leaf_cells(&GRAPH.read(), head);
    //     let mut corners = Vec::new();
    //     for (pointer, cell) in leaves {
    //         // This guy is a problem for us, my zorder doesn't have a way to condense to binary
    //         // currently
    //         // let zorder = ZorderPath::from_cell(cell.cell, start.height - cell.pointer.height);
    //         corners.push( Corners::new(
    //             cell_corners(cell.into(), pointer.height, min_cell_length),
    //             pointer.pointer,
    //             // if !BLOCKS.is_solid_index(*cell.pointer.pointer) { 0 } else { cell_corner_mask(start, zorder) }
    //         ));
    //     }
    //     corners 
    // }


}

// fn apply_drag() {
//     const DRAG_MULTIPLIER: f32 = 0.95;
//     for entity in &mut ENTITIES.write().entities { 
//         entity.velocity = (entity.velocity * DRAG_MULTIPLIER).snap_zero();
//         entity.angular_velocity = (entity.angular_velocity * DRAG_MULTIPLIER).snap_zero();
//     }
// }

// fn tick_entities(delta_tick: f32) {
//     for entity in &mut ENTITIES.write().entities {
//         entity.location.position += (entity.velocity * delta_tick).snap_zero();
//         entity.rel_rotate((entity.angular_velocity * delta_tick).snap_zero());
//     }
// }
// pub fn just_move() {
//     tick_entities(1.);
//     apply_drag();
// }

