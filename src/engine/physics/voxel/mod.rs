mod voxel_shape;
mod voxel_faces;
pub mod voxel_manifolds;

use crate::engine::grid::DagPointer;


pub struct Voxels {
    pub geometry: DagPointer,
    graph: crate::GRAPH,

    faces: Vec<voxel_faces::FaceNode>,
}
impl Voxels {
    pub fn new(geometry: DagPointer, graph: crate::GRAPH) -> Self {
        Self {
            graph,
            geometry,

            faces: Vec::new(),
        }
    }
}
