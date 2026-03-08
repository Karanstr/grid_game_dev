pub mod graph;
pub mod basic_node2d;

pub use graph::*;
pub use basic_node2d::*;

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
