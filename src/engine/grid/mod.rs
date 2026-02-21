mod graph;
mod basic_node2d;
mod partition;

pub use graph::{SparseDirectedGraph, Index, Node, GraphNode, Step};
pub use basic_node2d::{BasicNode2d, Zorder2d};
pub use partition::*;

use serde::{Deserialize, Serialize};
#[derive(Debug, Copy, Clone, Serialize, Deserialize, derive_new::new)]
pub struct ExternalPointer {
  pub pointer: Index,
  pub height: u32
}
