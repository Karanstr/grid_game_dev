use graph::Index;

pub mod graph;
pub mod basic_node2d;

pub use graph::*;
pub use basic_node2d::*;

#[derive(Debug, Copy, Clone)]
pub struct ExternalPointer {
    pub pointer : Index,
    pub height : u32
}
impl ExternalPointer {
    pub fn new(pointer: Index, height: u32) -> Self {
        Self {
            pointer,
            height,
        }
    }
}
