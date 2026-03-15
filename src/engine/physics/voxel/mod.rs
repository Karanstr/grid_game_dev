mod voxel_shape;
mod voxel_faces;
pub mod voxel_manifolds;

pub use crate::engine::{grid::dim2::{DagPointer, Cell}, physics::voxel::voxel_faces::Faces};
pub use voxel_manifolds::*;

pub struct Voxels {
    pub geometry: DagPointer,
    pub graph: crate::GRAPH,

    pub faces: Vec<(Faces, Cell)>,
}
impl Voxels {
    pub fn new(graph: crate::GRAPH) -> Self {
        Self {
            graph,
            geometry: DagPointer::new(0, 0),

            faces: Vec::new(),
        }
    }

    pub fn length(height: u32) -> f32 {
        2u32.pow(height) as f32
    }
}
