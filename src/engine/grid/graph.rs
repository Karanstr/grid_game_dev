use ahash::AHashMap;
use lilypads::Pond;

pub type Index = u32;

pub struct SparseDirectedGraph<const D: usize, Node: GraphNode<D>> {
  pub nodes : Pond<Node>,
  ref_count: Vec<u32>,
  index_lookup : AHashMap<Node, Index>,
}
impl<const D:usize, Node: GraphNode<D>> SparseDirectedGraph<D, Node> {
  pub fn new() -> Self {
    Self {
      nodes : Pond::new(),
      ref_count : Vec::new(),
      index_lookup : AHashMap::new(),
    }
  }

  /// Returns a trail with length path.len() + 1. trail.first() is the head of the trail and trail.last() is the node the path leads to.
  pub fn get_trail(&self, head: Index, path: &[Node::Children]) -> Vec<Index>  {
    let mut trail = Vec::with_capacity(path.len() + 1);
    trail.push(head);
    for step in 0 .. path.len() { trail.push(self.child(trail[step], path[step])) }
    trail
  }
  
  fn get_ref(&self, idx: Index) -> u32 { self.ref_count[idx as usize] }

  fn add_ref(&mut self, idx: Index) {
    if idx as usize >= self.ref_count.len() { self.ref_count.resize(idx as usize + 1, 0) }
    self.ref_count[idx as usize] += 1;
  }

  fn decrement_ref(&mut self, idx: Index) {
    let mut queue = vec![idx];
    while let Some(cur_idx) = queue.pop() {
      self.ref_count[cur_idx as usize] -= 1;
      if self.get_ref(cur_idx) == 0 {
        let old_node = self.nodes.free(cur_idx as usize).unwrap();
        self.index_lookup.remove(&old_node);
        for &child in Node::Children::all() {
          queue.push(old_node.get(child));
        }
      }
    }
  }
  
  /// Temporary function which adds a self-referencing node (a leaf)
  pub fn add_leaf(&mut self) -> Index {
    let idx = self.nodes.next_index() as Index;
    // Come back here!
    let mut leaf = Node::DEFAULT;
    for &child in Node::Children::all() {
      leaf = leaf.with_child(child, idx);
    }
    self.add_node(leaf)
  }

  fn add_node(&mut self, node: Node) -> Index {
    let idx = self.nodes.insert(node.clone()) as Index;
    for &child in Node::Children::all() { self.add_ref(node.get(child)); }
    self.index_lookup.insert(node, idx);
    idx
  }

  fn propagate_change(&mut self, path: &[Node::Children], trail: &[Index], mut new_child: Index) -> Index {
    for cur_depth in (0 .. path.len()).rev() {
      let new_node = self.node(trail[cur_depth]).with_child(path[cur_depth], new_child);
      new_child = if let Some(idx) = self.find_index(&new_node) { idx } else { self.add_node(new_node) };
    };
    new_child
  }

  pub fn set_node(&mut self, head:Index, path: &[Node::Children], new_idx:Index) -> Index {
    let trail = self.get_trail(head, path);
    if *trail.last().unwrap() == new_idx { return head }
    let new_head = self.propagate_change(path, &trail, new_idx);
    self.add_ref(new_head);
    self.decrement_ref(head);
    new_head
  }

  fn find_index(&self, node: &Node) -> Option<Index> { self.index_lookup.get(node).copied() }
  
  pub fn node(&self, idx:Index) -> &Node { self.nodes.get(idx as usize).unwrap() }

  pub fn child(&self, idx:Index, child: Node::Children) -> Index { self.node(idx).get(child) }

  /// Wraps [SparseDirectedGraph::get_trail] to function as a 'read end of path'
  pub fn descend(&self, head:Index, path:&[Node::Children]) -> Index { *self.get_trail(head, path).last().unwrap() }

  pub fn get_root(&mut self, idx:Index) -> Index { self.add_ref(idx); idx }

}

// impl<const D: usize, InternalNode: GraphNode<D>> SparseDirectedGraph<D, InternalNode> {
//
//   // Format: Leaves are not guaranteed a location, but we know if we walk forwards in the tree,
//   // we will always see dependencies before dependents. This means the head will always be the last node
//   pub fn export<N: Node<D>>(&self, head: Index) -> Vec<N> {
//     let mut export = Vec::new();
//     let mut remapped: AHashMap<Index, Index> = AHashMap::new();
//     // This works because, by stepping depthfirst, all child nodes are guaranteed to be after their
//     // parents. By reversing this, we guarantee seeing all children first.
//     for idx in bfs_nodes(&self.nodes.unsafe_data(), head).into_iter().rev() {
//       if remapped.contains_key(&idx) { continue } // If we've already catalogued this node
//       let old_node = self.nodes.get(idx as usize).unwrap();
//       let mut new_children = Vec::with_capacity(4);
//       for &child in InternalNode::Children::all() {
//         let old_child_idx = old_node.get(child);
//         // Slight edgecase to properly recreate self reference
//         new_children.push( if old_child_idx == idx { 
//           // The idx we'll be pushing this node into
//           export.len() as u32
//         } else {
//           *remapped.get(&old_node.get(child)).unwrap()
//         });
//       }
//       remapped.insert(idx, export.len() as u32);
//       export.push(N::new(&new_children));
//     }
//     export
//   }
//
//   /// remap should be populated with (ImportLeaf --> SelfLeaf)
//   /// output: object head
//   pub fn import(&mut self, data: Vec<InternalNode>, mut _remap: AHashMap<Index, Index>) -> Index {
//     for _node in data {
//       // Load it, 
//
//     }
//     todo!()
//   }
//
// }

use std::{collections::VecDeque, mem::MaybeUninit};
pub fn bfs_nodes<const D: usize, N: Node<D> >(nodes: &Vec<N>, head: Index) -> Vec<Index> {
  let mut queue = VecDeque::from([head]);
  let mut bfs_indexes = Vec::new();
  'next: while let Some(idx) = queue.pop_front() {
    bfs_indexes.push(idx);
    let node = nodes[idx as usize];
    for &child in N::Children::all() {
      let child_idx = node.get(child);
      // This is technically cheating, but for now if any child is a cycle the node is a leaf
      if child_idx == idx { continue 'next }
      queue.push_back(child_idx);
    }
  }
  bfs_indexes
}

pub trait Step<const D: usize>: std::fmt::Debug + Clone + Copy {
  // const COUNT: usize;

  fn all() -> &'static [Self];

  /// Subcell is a 1xD matrix representing which subcell the step will take us into.
  /// For a 2d ZORDER when Top Left is [0, 0], this means [0, 1] is Bottom Left, [1, 0] is Top
  /// Right, and [1, 1] is Bottom Right
  /// Requesting an invalid subcell returns None
  fn new(subcell: [u32; D]) -> Option<Self>;

  /// Returns an 1xD matrix representing the orthogonal coordinates
  /// of the subcell this step will take us to
  fn subcell(&self) -> [u32; D];

  /// Cell is a 1xD matrix representing the orthogonal coordinates of a cell n steps downward.
  /// This function removes the last step from the coordinates, mutating them into coordinates to
  /// the supercell n - 1 steps downward and returning the [Step] which would take that supercell
  /// to the provided cell.
  fn extract_last_step(cell: &mut [u32; D]) -> Option<Self>;

  /// Cell takes a 1xT::DIMS matrix representing the cell we want to reach.
  /// Depth is the number of steps we want to get there.
  /// This means we can access any cell in a region with side lengths 2^Depth.
  /// Attempting to access a cell outside of this defined region returns None
  fn path_from_cell(mut cell: [u32; D], depth: u32) -> Option< Vec<Self> > {
    if *cell.iter().max().unwrap() >= 1 << depth { eprintln!("Cell is too large for depth {depth}"); return None; }
    let mut path = Vec::with_capacity(depth as usize);
    for _ in 0 .. depth { path.push(Self::extract_last_step(&mut cell)?); }
    path.reverse();
    Some(path)
  }

  fn path_to_cell(path: &[Self]) -> [u32; D];
}

// Nodes are anything with valid children access
pub trait Node<const D: usize>: std::fmt::Debug + Clone + Copy {
  type Children: Step<D> + 'static;
  const DEFAULT: Self;
  fn get(&self, child: Self::Children) -> Index;
  fn with_child(&self, child: Self::Children, idx: Index) -> Self;
}
// GraphNodes are nodes which can be hashed, making them valid for SDG storage
pub trait GraphNode<const D: usize>: Node<D> + std::hash::Hash + Eq {}

// HACK: I hate this, but its what we're doing for now
impl<T, const D: usize> Node<D> for MaybeUninit<T> where T: Node<D> {
  type Children = T::Children;
  const DEFAULT: Self = MaybeUninit::new(T::DEFAULT);
  /// UNSAFE!! YOU MUST PROMISE THIS NODE IS INITIALIZED
  fn get(&self, child: Self::Children) -> Index {
    unsafe { self.assume_init_ref() }.get(child)
  }
  /// UNSAFE!! YOU MUST PROMISE THIS NODE IS INITIALIZED
  fn with_child(&self, child: Self::Children, idx: Index) -> Self {
    MaybeUninit::new(unsafe { self.assume_init_read() }.with_child(child, idx) )
  }
}

