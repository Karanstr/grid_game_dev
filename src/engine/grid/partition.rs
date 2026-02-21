use glam::{Vec2, UVec2};
use crate::engine::{entities::Location, grid::*};
// use crate::engine::entities::Location;
// use crate::globals::GRAPH;
//Value loosely tuned to prevent both phasing and catching on corners
//Used to sample area around a point to determine what cell(s) it's in
// pub const LIM_OFFSET: f32 = 2. / 0xFFFF as f32;

// #[derive(Debug, Clone, Copy, )]
// pub struct CellData {
    // pub pointer : ExternalPointer,
    // pub cell : UVec2,
// }
// impl CellData {
//     pub fn bound_data(&self) -> (Vec2, u32) { (self.cell.as_vec2(), self.pointer.height) }
//
//     pub fn to_point(&self, location:Location, min_cell_length:Vec2) -> Vec2 {
//         let cell_top_left = self.cell.as_vec2() * cell_length(self.pointer.height, min_cell_length); 
//         let global_position = location.position - center_to_edge(location.pointer.height, min_cell_length);
//         cell_top_left + global_position + center_to_edge(self.pointer.height, min_cell_length)
//     }
// }

pub fn cell_length(height:u32, min_cell_length:Vec2) -> Vec2 {
    min_cell_length * 2_f32.powi(height as i32)
}

pub fn center_to_edge(height:u32, min_cell_length:Vec2) -> Vec2 {
    cell_length(height, min_cell_length) / 2.
}


pub fn dfs_leaf_cells<const D: usize, N: GraphNode<D>>(graph: &SparseDirectedGraph<D, N>, head: ExternalPointer) -> Vec<(ExternalPointer, [u32; D])> {
    let mut stack = Vec::from([(
        head.pointer,
        N::Children::path_from_cell([0; D], 0).unwrap()
    )]);
    let mut leaves = Vec::new();
    while let Some((pointer, path)) = stack.pop() {
        // if self.is_leaf(pointer) {
        if pointer < 4 {
            leaves.push((
                ExternalPointer::new( pointer, head.height - path.len() as u32), 
                N::Children::path_to_cell(&path)
            ));
        } else { 
            for &child in N::Children::all() {
                let node = graph.nodes.get(pointer as usize).unwrap();
                let mut child_path = path.clone();
                child_path.push(child);
                stack.push((node.get(child), child_path));
            }
        }
    }
    leaves
}


pub fn point_to_cell(location: Location, height:u32, point:Vec2) -> Option<UVec2> {
    // let mut surrounding = [None; 4];
    let grid_length = cell_length(location.pointer.height, location.min_cell_length);
    let cell_length = cell_length(height, location.min_cell_length);
    let origin_position = point - (location.position - grid_length / 2.);
    if origin_position.clamp(Vec2::ZERO, grid_length) == origin_position {
        Some( (origin_position / cell_length).floor().as_uvec2() )
    } else { None }
}

// pub mod gate {
//     use glam::UVec2;
//     use macroquad::math::UVec2 as MUVec2;
//     use sdg::basic2d::{Geometry2d, Zorder2d};
//
//     use super::*;
//     pub fn point_to_cells(location:Location, height:u32, point:Vec2) -> [Option<MUVec2>; 4]{
//         let mut surrounding = [None; 4];
//         let grid_length = cell_length(location.pointer.height, location.min_cell_length);
//         let cell_length = cell_length(height, location.min_cell_length);
//         let origin_position = point - (location.position - grid_length / 2.);
//         let directions = [
//             Vec2::new(-1., -1.),
//             Vec2::new(1., -1.),
//             Vec2::new(-1., 1.),
//             Vec2::new(1., 1.),
//         ];
//         for i in 0 .. 4 {
//             let cur_point = origin_position + LIM_OFFSET * directions[i];
//             if cur_point.clamp(Vec2::ZERO, grid_length).approx_eq(cur_point) {
//                 surrounding[i] = Some( (cur_point / cell_length).floor().as_uvec2() )
//             }
//         }
//         surrounding
//     }
//
//     pub fn point_to_real_cells(location:Location, point:Vec2) -> [Option<CellData>; 4] {
//         let mut surrounding = [None; 4];
//         let cells = point_to_cells(location, 0, point);
//         for i in 0..4 {
//             if let Some(cell) = cells[i] {
//                 surrounding[i] = Some(find_real_cell(location.pointer, UVec2::new(cell.x, cell.y)));
//             }
//         }
//         surrounding
//     }
//
//     /// Only works if cell is at height 0
//     pub fn find_real_cell(head: ExternalPointer, cell: UVec2) -> CellData {
//         // let path = ZorderPath::from_cell(cell, start.height);
//         let path = Zorder2d::path(UVec2::new(cell.x, cell.y), start.height);
//         let trail = GRAPH.read().get_trail(head.pointer, &path);
//         let pointer = GRAPH.read().descend(start.pointer, &path);
//         // THIS IS WRONG, I NEED A WAY TO KNOW HOW DEEP WE GO BEFORE WE START LOOPING!!
//         let zorder = path.with_depth(start.height - pointer.height);
//         CellData::new(pointer, zorder.to_cell())
//     }
//
// }

