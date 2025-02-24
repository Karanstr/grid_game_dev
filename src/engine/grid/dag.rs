use std::collections::{HashMap, VecDeque};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use vec_mem_heap::prelude::{NodeField, AccessError};
pub use vec_mem_heap::Index;

pub trait GraphNode : Node + std::fmt::Debug + Clone + std::hash::Hash + Eq {}

#[derive(Debug, Copy, Clone, Serialize, Deserialize, derive_new::new)]
pub struct ExternalPointer {
    pub pointer : Index,
    pub height : u32
}

pub trait Node : Clone {
    fn new(children:[Index; 4]) -> Self;
    fn children(&self) -> [Index; 4];
    fn set_child(&mut self, child:usize, index:Index);
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BasicNode {
    children : [Index; 4],
}
impl Node for BasicNode {
    fn new(children:[Index; 4]) -> Self { Self { children } }
    fn children(&self) -> [Index; 4] { self.children }
    fn set_child(&mut self, child:usize, index:Index) { self.children[child] = index }
}
impl GraphNode for BasicNode {}
impl<T> Node for vec_mem_heap::internals::MemorySlot<T> where T: GraphNode {
    fn new(_:[Index; 4]) -> Self { panic!("Don't do that!") }
    fn children(&self) -> [Index; 4] {
        match self.unwrap_steward() {
            Ok(steward) => { steward.data.children() }
            Err(_) => { panic!("Attempted use after free") }
        }
    }
    fn set_child(&mut self, child:usize, index:Index) {
        match self.unwrap_steward_mut() {
            Ok(steward) => { steward.data.set_child(child, index) }
            Err(err) => { panic!("{err:?}") }
        }
    }
}
pub struct SparseDirectedGraph<T: GraphNode> {
    pub nodes : NodeField<T>,
    pub index_lookup : HashMap<T, Index>,
    leaf_count : u8, 
}
impl<T: GraphNode> SparseDirectedGraph<T> {
    //Utility
    pub fn new(leaf_count:u8) -> Self {
        let mut instance = Self {
            nodes : NodeField::new(),
            index_lookup : HashMap::new(),
            leaf_count
        };
        for i in 0 .. leaf_count {
            instance.add_node(T::new([Index(i as usize); 4]));
        }
        instance
    }
    
    pub fn is_leaf(&self, index:Index) -> bool {
        *index < self.leaf_count as usize
    }

    fn get_trail(&self, start:Index, path:&[u32]) -> Vec<Index>  {
        let mut trail = vec![start];
        for step in 0 .. path.len() {
            let parent = trail[step];
            match self.child(parent, path[step] as usize) {
                Ok( child ) if child != parent => trail.push(child),
                Ok(_) => break,
                Err(error) => panic!("Trail encountered a fatal error, {error:?}")
            };
        }
        trail 
    }

    //Private functions used for writing
    fn find_index(&self, node:&T) -> Option<Index> {
        self.index_lookup.get(node).copied()
    }

    fn add_node(&mut self, node:T) -> Index {
        let index = self.nodes.push(node.clone());
        self.index_lookup.insert(node, index);
        index
    }

    fn propagate_change(
        &mut self,
        path: &[u32],
        trail: &[Index],
        mut cur_pointer: ExternalPointer,
        mut old_parent: Index,
    ) -> Result<(Index, ExternalPointer, Option<T>), AccessError> {
        for cur_depth in (0 .. path.len()).rev() {
            //Trailing off early means we're at a leaf, we can just repeat that leaf and get sparsity by default
            old_parent = if cur_depth < trail.len() { trail[cur_depth] } else { *trail.last().unwrap() };
            let new_parent_node =  {
                let mut new_parent = self.node(old_parent)?.clone();
                new_parent.set_child(path[cur_depth] as usize, cur_pointer.pointer);
                new_parent
            };
            cur_pointer.height += 1;
            cur_pointer.pointer = match self.find_index(&new_parent_node) {
                Some(pointer) => pointer,
                None => {
                    if self.nodes.status(old_parent).unwrap().get() == 2 && !self.is_leaf(old_parent) {
                        cur_pointer.pointer = old_parent;
                        return Ok((old_parent, cur_pointer, Some(new_parent_node)))
                    } else { self.add_node(new_parent_node) }
                }
            };
        };
        Ok((old_parent, cur_pointer, None))
    }

    //Public functions used for writing
    pub fn set_node(&mut self, start:ExternalPointer, path:&[u32], new_pointer:Index) -> Result<ExternalPointer, AccessError> {
        if let Some(pointer) = self.read(start, path) { 
            if pointer.pointer == new_pointer { return Ok(start) }
        } else { return Err(AccessError::OperationFailed) }
        let trail = self.get_trail(start.pointer, path);
        let (old_parent, cur_pointer, early_node) = self.propagate_change(
            path,
            &trail[..],
            ExternalPointer::new(new_pointer, start.height - path.len() as u32),
            start.pointer,
        )?;
        let last_leaf = self.leaf_count as usize - 1;
        let old_nodes = bfs_nodes(self.nodes.internal_memory(), old_parent, last_leaf); 
        let early_exit = if let Some(node) = early_node {
            self.index_lookup.remove(&self.nodes.replace(old_parent, node.clone()).unwrap());
            self.index_lookup.insert(node, old_parent);
            true
        } else { false };
        for index in bfs_nodes(self.nodes.internal_memory(), cur_pointer.pointer, last_leaf) {
            self.nodes.add_ref(index).unwrap()
        }
        self.mass_remove(&old_nodes);
        // Returning start because the root node never changes
        if early_exit { Ok(start) } else { Ok(cur_pointer) }
    }

    pub fn mass_remove(&mut self, indices:&[Index]) {
        for index in indices {
            self.nodes.remove_ref(*index).unwrap();
            if self.nodes.status(*index).unwrap().get() == 1 && !self.is_leaf(*index) {
                self.index_lookup.remove(&self.nodes.remove_ref(*index).unwrap().unwrap());
            }
        }
    }

    //Public functions used for reading
    pub fn node(&self, pointer:Index) -> Result<&T, AccessError> {
        self.nodes.data(pointer)
    }

    pub fn child(&self, node:Index, zorder:usize) -> Result<Index, AccessError> {
        Ok( self.node(node)?.children()[zorder] )
    }
    
    pub fn read(&self, start:ExternalPointer, path:&[u32]) -> Option<ExternalPointer> {
        let trail = self.get_trail(start.pointer, path);
        let node_pointer = trail.last()?;
        Some(ExternalPointer::new(*node_pointer, start.height - (trail.len() as u32 - 1)))
    }

    pub fn get_root(&mut self, leaf:usize, height:u32) -> ExternalPointer {
        self.nodes.add_ref(Index(leaf)).unwrap();
        ExternalPointer::new(Index(leaf), height)
    }

}

#[derive(Serialize, Deserialize)]
struct TreeStorage<T : GraphNode> {
    root: ExternalPointer,
    nodes: Vec<T>,
}
//Assumes constant leaf count. Eventually add more metadata
impl<T: GraphNode + Serialize + DeserializeOwned> SparseDirectedGraph<T> {
    pub fn save_object_json(&self, start:ExternalPointer) -> String {
        let mut object_graph = Self::new(self.leaf_count);
        let root_index = object_graph.clone_graph(self.nodes.internal_memory(), start.pointer);
        serde_json::to_string(&TreeStorage {
            root : ExternalPointer::new(root_index, start.height),
            nodes : object_graph.nodes.internal_memory().iter().map(|node| T::new(node.children())).collect(), 
        }).unwrap()
    }
    
    //Currently requires the nodetype of both graph and data to be the same.
    pub fn load_object_json(&mut self, json:String) -> ExternalPointer {
        let temp:TreeStorage<T> = serde_json::from_str(&json).unwrap();
        ExternalPointer::new(self.clone_graph(&temp.nodes, temp.root.pointer), temp.root.height)
    }

    // Clippy thinks I should pass a slice here instead of a vector, but passing a partial slice is very likely to lead to operation failure
    //Assumes equal leaf count (between the two graphs)
    fn clone_graph<N : Node> (&mut self, from:&Vec<N>, start:Index) -> Index {
        let mut remapped = HashMap::new();
        for i in 0 .. self.leaf_count as usize { remapped.insert(Index(i), Index(i)); }
        for pointer in bfs_nodes(from, start, self.leaf_count as usize - 1).into_iter().rev() {
            if !remapped.contains_key(&pointer) {
                let old_kids = &from[*pointer].children();
                let new_node = T::new([
                    *remapped.get(&old_kids[0]).unwrap(),
                    *remapped.get(&old_kids[1]).unwrap(),
                    *remapped.get(&old_kids[2]).unwrap(),
                    *remapped.get(&old_kids[3]).unwrap()
                ]);
                remapped.insert(pointer, self.add_node(new_node));
            }
            self.nodes.add_ref(*remapped.get(&pointer).unwrap()).unwrap();
        }
        *remapped.get(&start).unwrap()
    }

}

// Assumes leaves are stored contiguously at the front of the slice.
pub fn bfs_nodes<N: Node>(nodes:&Vec<N>, start:Index, last_leaf:usize) -> Vec<Index> {
    let mut queue = VecDeque::from([start]);
    let mut bfs_indexes = Vec::new();
    while let Some(index) = queue.pop_front() {
        bfs_indexes.push(index);
        if *index > last_leaf { queue.extend(nodes[*index].children()) }
    }
    bfs_indexes
}

