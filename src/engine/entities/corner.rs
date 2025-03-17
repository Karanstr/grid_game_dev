use crate::engine::grid::dag::{Index, ExternalPointer};
use crate::engine::grid::partition::{ZorderPath, CellData, cell_length};
use macroquad::math::{Vec2, IVec2};
use crate::globals::*;

// Extract corners into our collision detection system (once we write it)
#[derive(Debug, Clone, derive_new::new)]
pub struct Corners {
    pub points : [Vec2; 4],
    pub index : Index,
    pub mask : u8,
}

fn cell_corner_mask(start: ExternalPointer, zorder: ZorderPath) -> u8 {
    const CORNER_CHECKS: [([(IVec2, u8); 3], u8); 4] = [
        // Format: ([(offset, step_direction), ...], corner_mask_bit)
        ([(IVec2::new(-1, 0), 0b01), (IVec2::new(0, -1), 0b10), (IVec2::new(-1, -1), 0b11)], 0b0001), // Top Left
        ([(IVec2::new(1, 0), 0b00), (IVec2::new(0, -1), 0b11), (IVec2::new(1, -1), 0b10)], 0b0010),  // Top Right
        ([(IVec2::new(-1, 0), 0b11), (IVec2::new(0, 1), 0b00), (IVec2::new(-1, 1), 0b01)], 0b0100),  // Bottom Left
        ([(IVec2::new(1, 0), 0b10), (IVec2::new(0, 1), 0b01), (IVec2::new(1, 1), 0b00)], 0b1000),    // Bottom Right
    ];

    let mut exposed_mask = 0b0000;
    'corner: for (checks, mask) in CORNER_CHECKS {
        for (offset, direction) in checks {
            let Some(mut check_zorder) = zorder.move_cartesianly(offset) else { continue };
            for _ in 0 .. start.height - check_zorder.depth {
                check_zorder = check_zorder.step_down(direction as u32)
            }
            let pointer = GRAPH.read().read(start, &check_zorder.steps()).unwrap();
            if BLOCKS.is_solid_index(*pointer.pointer) { continue 'corner }
        }
        exposed_mask |= mask;
    }
    exposed_mask
}

//The top left corner of the root is (0, 0)
fn cell_corners(cell:CellData, min_cell_length:Vec2) -> [Vec2; 4] {
    let cell_size = cell_length(cell.pointer.height, min_cell_length);
    let top_left_corner = cell.cell.as_vec2() * cell_size;
    [
        top_left_corner,
        top_left_corner.with_x(top_left_corner.x + cell_size.x),
        top_left_corner.with_y(top_left_corner.y + cell_size.y),
        top_left_corner + cell_size,
    ]
}

pub fn tree_corners(start:ExternalPointer, min_cell_length:Vec2) -> Vec<Corners> {
    let leaves = GRAPH.read().dfs_leaf_cells(start);
    let mut corners = Vec::new();
    for cell in leaves {
        let zorder = ZorderPath::from_cell(cell.cell, start.height - cell.pointer.height);
        corners.push( Corners::new(
            cell_corners(cell, min_cell_length),
            cell.pointer.pointer,
            if !BLOCKS.is_solid_index(*cell.pointer.pointer) { 0 } else { cell_corner_mask(start, zorder) }
        ));
    }
    corners 
}
