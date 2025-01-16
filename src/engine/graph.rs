use std::collections::{HashMap, VecDeque};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
pub use vec_mem_heap::{NodeField, Index, prelude::AccessError};
use std::hash::Hash;
use derive_new::new;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, new)]
pub struct ExternalPointer {
    pub pointer : Index,
    pub height : u32
}

pub trait GraphNode : Clone {
    fn new(children:[Index; 4]) -> Self;
    fn children(&self) -> [Index; 4];
    fn set_child(&mut self, child:usize, index:Index);
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Node {
    children : [Index; 4],
}
impl GraphNode for Node {
    fn new(children:[Index; 4]) -> Self { Self { children } }
    fn children(&self) -> [Index; 4] { self.children }
    fn set_child(&mut self, child:usize, index:Index) { self.children[child] = index }
}
impl<T> GraphNode for vec_mem_heap::internals::MemorySlot<T> where T: GraphNode{
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
pub struct SparseDirectedGraph<T: GraphNode + Hash + Eq> {
    pub nodes : NodeField<T>,
    pub index_lookup : HashMap<T, Index>,
    leaf_count : u8, 
}
impl<T: GraphNode + Hash + Eq> SparseDirectedGraph<T> {
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

    //Public functions used for writing
    //This function is so big, any chance we can make it smaller?
    pub fn set_node(&mut self, start:ExternalPointer, path:&[u32], new_pointer:Index) -> Result<ExternalPointer, AccessError> {
        if let Some(pointer) = self.read(start, path) { 
            if pointer.pointer == new_pointer { return Ok(start) }
        } else { return Err(AccessError::OperationFailed) }
        let trail = self.get_trail(start.pointer, path);
        let mut cur_pointer = ExternalPointer::new(new_pointer, start.height - path.len() as u32);
        let mut old_parent = new_pointer;
        let mut early_exit = false;
        let mut early_node = None;
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
                        early_node = Some(new_parent_node);
                        cur_pointer.pointer = old_parent;
                        early_exit = true;
                        break
                    } else { self.add_node(new_parent_node) }
                }
            };
        }
        let last_leaf = self.leaf_count as usize - 1;
        let old_nodes = bfs_nodes(self.nodes.internal_memory(), old_parent, last_leaf); 
        if let Some(node) = early_node {
            let old_node = self.nodes.replace(old_parent, node.clone()).unwrap();
            self.index_lookup.remove(&old_node);
            self.index_lookup.insert(node, old_parent);
        } 
        for index in bfs_nodes(self.nodes.internal_memory(), cur_pointer.pointer, last_leaf) {  self.nodes.add_ref(index).unwrap() }
        for index in old_nodes {
            self.nodes.remove_ref(index).unwrap();
            if self.nodes.status(index).unwrap().get() == 1 && !self.is_leaf(index) {
                self.index_lookup.remove(&self.nodes.remove_ref(index).unwrap().unwrap());
            }
        }
        if early_exit { Ok(start) } else { Ok(cur_pointer) }
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
        let Some(node_pointer) = trail.last() else { return None };
        Some(ExternalPointer::new(*node_pointer, start.height - (trail.len() as u32 - 1)))
    }

    pub fn get_root(&mut self, leaf:usize, height:u32) -> ExternalPointer {
        self.nodes.add_ref(Index(leaf)).unwrap();
        ExternalPointer::new(Index(leaf), height)
    }

}

#[derive(Serialize, Deserialize)]
struct TreeStorage<T : GraphNode> {
    nodes: Vec<T>,
    root: ExternalPointer,
}
//Assumes constant leaf count. Eventually add more metadata
impl<T: GraphNode + Clone + Hash + Eq + Serialize + DeserializeOwned> SparseDirectedGraph<T> {
    pub fn save_object_json(&self, start:ExternalPointer) -> String {
        let mut object_graph = Self::new(self.leaf_count);
        let root_index = object_graph.clone_graph(self.nodes.internal_memory(), start.pointer);
        serde_json::to_string_pretty(&TreeStorage {
            nodes : object_graph.nodes.internal_memory().iter().map(|node| T::new(node.children())).collect(), 
            root : ExternalPointer::new(root_index, start.height)
        }).unwrap()
    }
    
    //Currently requires the nodetype of both graph and data to be the same.
    pub fn load_object_json(&mut self, json:String) -> ExternalPointer {
        let temp:TreeStorage<T> = serde_json::from_str(&json).unwrap();
        ExternalPointer::new(self.clone_graph(&temp.nodes, temp.root.pointer), temp.root.height)
    }

    //Assumes equal leaf count (between the two graphs)
    fn clone_graph<N : GraphNode> (&mut self, from:&Vec<N>, start:Index) -> Index {
        let mut remapped = HashMap::new();
        for i in 0 .. self.leaf_count as usize { remapped.insert(Index(i), Index(i)); }
        for pointer in bfs_nodes(from, start, self.leaf_count as usize).into_iter().rev() {
            if !remapped.contains_key(&pointer) {
                let old_kids = &from[*pointer].children();
                let new_node = T::new([
                    *remapped.get(&old_kids[0]).unwrap(),
                    *remapped.get(&old_kids[1]).unwrap(),
                    *remapped.get(&old_kids[2]).unwrap(),
                    *remapped.get(&old_kids[3]).unwrap()
                ]);
                remapped.insert(pointer, self.add_node(new_node));
            } else { self.nodes.add_ref(*remapped.get(&pointer).unwrap()); }
        }
        *remapped.get(&start).unwrap()
    }

}

//Assumes leaves are stored contiguously at the front of the slice.
pub fn bfs_nodes<N: GraphNode>(nodes:&Vec<N>, start:Index, last_leaf:usize) -> Vec<Index> {
    let mut queue = VecDeque::from([start]);
    let mut bfs_indexes = Vec::new();
    while let Some(index) = queue.pop_front() {
        bfs_indexes.push(index);
        if *index > last_leaf { queue.extend(nodes[*index].children()) }
    }
    bfs_indexes
}