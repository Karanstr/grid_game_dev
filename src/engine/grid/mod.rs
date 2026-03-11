pub mod graph;
pub mod basic_node2d;

pub use graph::*;
pub use basic_node2d::*;

pub mod dim2 {
    pub use super::graph::*;
    pub type Graph2D<Node: GraphNode<2>> = SparseDirectedGraph<2, Node>;
    pub use super::basic_node2d::*;
    pub use super::DagPointer;
}

#[derive(Debug, Copy, Clone)]
pub struct DagPointer {
    pub head: Index,
    pub height: u32
}
impl DagPointer {
    pub fn new(head: Index, height: u32) -> Self {
        Self {
            head,
            height,
        }
    }
}
