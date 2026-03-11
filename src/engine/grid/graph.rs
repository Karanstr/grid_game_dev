use ahash::AHashMap;
use lilypads::Pond;

pub type Index = u32;

pub struct SparseDirectedGraph<const DIM: usize, Node: GraphNode<DIM>> {
  pub nodes : Pond<Node>,
  ref_count: Vec<u32>,
  index_lookup : AHashMap<Node, Index>,
}
impl<const DIM: usize, Node: GraphNode<DIM>> SparseDirectedGraph<DIM, Node> {
  pub fn new() -> Self {
    Self {
      nodes : Pond::new(),
      ref_count : Vec::new(),
      index_lookup : AHashMap::new(),
    }
  }

  /// Returns a trail with length path.len() + 1. trail.first() is the head of the trail and trail.last() is the node the path leads to.
  pub fn get_trail<P: Path<DIM, impl Step<DIM>>>(&self, head: Index, path: &P) -> Vec<Index>  {
    let mut trail = Vec::with_capacity((path.len() + 1) as usize);
    trail.push(head);
    for step in 0 .. path.len() { 
      trail.push(self.child(trail[step as usize], path.step_at(step as usize).unwrap() ))
    }
    trail
  }
  
  fn get_ref(&self, idx: Index) -> u32 { self.ref_count[idx as usize] }

  fn add_ref(&mut self, idx: Index) {
    if idx as usize >= self.ref_count.len() { self.ref_count.resize(idx as usize + 1, 0) }
    self.ref_count[idx as usize] += 1;
  }

  /// Relies on child count being 
  fn decrement_ref(&mut self, idx: Index) {
    let mut queue = vec![idx];
    while let Some(cur_idx) = queue.pop() {
      self.ref_count[cur_idx as usize] -= 1;
      if self.get_ref(cur_idx) == 0 {
        let old_node = self.nodes.free(cur_idx as usize).unwrap();
        self.index_lookup.remove(&old_node);
        for &child in Node::NativeStep::all() {
          queue.push(old_node.child(child));
        }
      }
    }
  }
  
  /// Temporary function which adds a self-referencing node (a leaf)
  pub fn add_leaf(&mut self) -> Index {
    let idx = self.nodes.next_index() as Index;
    // Come back here!
    let mut leaf = Node::DEFAULT;
    for &child in Node::NativeStep::all() {
      leaf = leaf.with_child(child, idx);
    }
    self.add_node(leaf)
  }

  fn add_node(&mut self, node: Node) -> Index {
    let idx = self.nodes.insert(node.clone()) as Index;
    for &child in Node::NativeStep::all() { self.add_ref(node.child(child)); }
    self.index_lookup.insert(node, idx);
    idx
  }

  fn propagate_change<P: Path<DIM, impl Step<DIM>>>(&mut self, path: &P, trail: &[Index], mut new_child: Index) -> Index {
    for cur_depth in (0 .. path.len()).rev() {
      let new_node = self.node(trail[cur_depth]).with_child(path.step_at(cur_depth).unwrap(), new_child);
      new_child = if let Some(idx) = self.find_index(&new_node) { idx } else { self.add_node(new_node) };
    };
    new_child
  }

  pub fn set_node<P: Path<DIM, impl Step<DIM>>>(&mut self, head:Index, path: &P, new_idx:Index) -> Index {
    let trail = self.get_trail(head, path);
    if *trail.last().unwrap() == new_idx { return head }
    let new_head = self.propagate_change(path, &trail, new_idx);
    self.add_ref(new_head);
    self.decrement_ref(head);
    new_head
  }

  fn find_index(&self, node: &Node) -> Option<Index> { self.index_lookup.get(node).copied() }
  
  pub fn node(&self, idx:Index) -> &Node { self.nodes.get(idx as usize).unwrap() }

  pub fn child(&self, idx:Index, child: impl Step<DIM>) -> Index { self.node(idx).child(child) }

  /// Wraps [SparseDirectedGraph::get_trail] to function as a 'read end of path'
  pub fn descend<P: Path<DIM, impl Step<DIM>>>(&self, head:Index, path: &P) -> Index { 
    *self.get_trail(head, path).last().unwrap()
  }

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
pub fn bfs_nodes<const D: usize, N: Node<D>>(nodes: &Vec<N>, head: Index) -> Vec<Index> {
  let mut queue = VecDeque::from([head]);
  let mut bfs_indexes = Vec::new();

  'next: while let Some(idx) = queue.pop_front() {
    bfs_indexes.push(idx);
    let node = nodes[idx as usize];
    for &child in N::NativeStep::all() {
      let child_idx = node.child(child);
      // This is technically cheating, but for now if any child is a cycle the node is a leaf
      if child_idx == idx { continue 'next }
      queue.push_back(child_idx);
    }
  }

  bfs_indexes
}

pub fn dfs_leaves<const D: usize, N, S, P>(
  nodes: &Vec<N>, 
  head: Index
) -> Vec<(u32, P)>
where
  N: Node<D>,
  S: Step<D>,
  P: Path<D, S>
{
  let mut stack = vec![(head as usize, P::default())];
  let mut leaves = Vec::new();

  'next: while let Some((idx, zorder)) = stack.pop() {
    let cur_node = nodes[idx];
    for &child in S::all().iter().rev() {
      let child_idx = cur_node.child(child);
      // This is kinda cheating but it's ok
      if child_idx == idx as u32 {
        leaves.push((idx as u32, zorder));
        continue 'next
      }
      let mut child_zorder = zorder.clone();
      child_zorder.push_step(child);
      stack.push((child_idx as usize, child_zorder));
    }
  }

  leaves
}

/// Represents a single step into one of the children of a node
pub trait Step<const DIM: usize>: std::fmt::Debug + Clone + Copy + 'static {
  fn all() -> &'static [Self];

  /// Not sure if I love this.
  /// I need a single persistant mapping per Step impl for Node to index with
  fn as_usize(&self) -> usize;
}

pub trait Path<const DIM: usize, S: Step<DIM>>: Default + Clone {
  type InternalStep;
  fn to_internal(step: S) -> Self::InternalStep;
  fn from_internal(step: Self::InternalStep) -> S;
  /// Not returning Result is bad interface design.
  /// We'll see when I decide to fix that.
  fn push_internal(&mut self, step: Self::InternalStep);
  fn pop_internal(&mut self) -> Option<Self::InternalStep>;
  
  fn push_step(&mut self, step: S) {
    let step = Self::to_internal(step);
    self.push_internal(step);
  }
  fn pop_step(&mut self) -> Option<S> {
    let step = self.pop_internal()?;
    Some(Self::from_internal(step))
  }

  fn len(&self) -> usize;
  fn step_at(&self, n: usize) -> Option<S>;

  fn convert<P: Path<DIM, S> + Default>(&self) -> P {
      let mut result = P::default();
      for i in 0 .. self.len() {
          result.push_step(self.step_at(i).unwrap());
      }
      result
  }
}
impl<const DIM: usize, S: Step<DIM>> Path<DIM, S> for Vec<S> {
    type InternalStep = S;
    fn to_internal(step: S) -> Self::InternalStep { step }
    fn from_internal(step: S) -> S { step }
    fn push_internal(&mut self, step: Self::InternalStep) {
        self.push(step);
    }
    fn pop_internal(&mut self) -> Option<Self::InternalStep> {
        self.pop()
    }

    fn len(&self) -> usize { self.len() }
    fn step_at(&self, n: usize) -> Option<S> { self.get(n).copied() }

}

// Nodes are anything with valid children access
/// DIM is the number of dimensions it exists in.
pub trait Node<const DIM: usize>: std::fmt::Debug + Clone + Copy {
  const DEFAULT: Self;
  /// The step operations should default to when I can't be assed to pass it over another way
  type NativeStep: Step<DIM>;

  fn child(&self, child: impl Step<DIM>) -> Index;
  fn with_child(&self, child: impl Step<DIM>, idx: Index) -> Self;
}

// GraphNodes are nodes which can be hashed, making them valid for SDG storage
pub trait GraphNode<const DIM: usize>: Node<DIM> + std::hash::Hash + Eq {}

// HACK: I hate this, but its what we're doing for now
impl<T, const DIM: usize> Node<DIM> 
for MaybeUninit<T> where T: Node<DIM> {
  const DEFAULT: Self = MaybeUninit::new(T::DEFAULT);
  type NativeStep = T::NativeStep;

  /// UNSAFE!! YOU MUST PROMISE THIS NODE IS INITIALIZED
  fn child(&self, child: impl Step<DIM>) -> Index {
    unsafe { self.assume_init_ref() }.child(child)
  }
  /// UNSAFE!! YOU MUST PROMISE THIS NODE IS INITIALIZED
  fn with_child(&self, child: impl Step<DIM>, idx: Index) -> Self {
    MaybeUninit::new(unsafe { self.assume_init_read() }.with_child(child, idx) )
  }
}

