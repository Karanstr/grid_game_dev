use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
pub use vec_mem_heap::{NodeField, Index, prelude::AccessError};
use std::hash::Hash;
use derive_new::new;

//Replace memheap index with a generic, making things simpler

#[derive(Debug, Copy, Clone, Serialize, Deserialize, new)]
pub struct ExternalPointer {
    pub pointer : Index,
    pub height : u32
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, new)]
pub struct Node {
    pub children : [Index; 4],
}

pub struct SparseDirectedGraph {
    pub nodes : NodeField<Node>,
    pub index_lookup : HashMap<Node, Index>,
    leaf_count : u8, 
}
impl SparseDirectedGraph {
    //Utility
    pub fn new(leaf_count:u8) -> Self {
        let mut instance = Self {
            nodes : NodeField::new(),
            index_lookup : HashMap::new(),
            leaf_count
        };
        for i in 0 .. leaf_count {
            instance.add_node(Node { children : [Index(i as usize); 4] });
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

    pub fn profile(&self) {
        println!("---------- STATUS ----------");
        let mut free_memory = 0;
        let mut reserved_memory = 0;
        for index in 0 .. self.nodes.internal_memory().len() {
            let cur_index = Index(index);
            if let Ok(status) = self.nodes.status(cur_index) {
                reserved_memory += 1;
            } else { free_memory += 1 }
        }

        println!("There are {} slots within the graph, consisting of:", reserved_memory);
        println!("{} free slots", free_memory);
    }

    //Private functions used for writing
    fn find_index(&self, node:&Node) -> Option<Index> {
        self.index_lookup.get(node).copied()
    }

    fn add_node(&mut self, node:Node) -> Index {
        let index = self.nodes.push(node.clone());
        self.index_lookup.insert(node, index);
        index
    }

    //Public functions used for writing
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
            //If it trails off early we know it's at a leaf so we can just repeat that leaf
            old_parent = if cur_depth < trail.len() { trail[cur_depth] } else { *trail.last().unwrap() };
            let new_parent_node =  {
                let mut new_parent = self.node(old_parent)?.clone();
                new_parent.children[path[cur_depth] as usize] = cur_pointer.pointer;
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
        let old_nodes = self.bfs_nodes(ExternalPointer::new(old_parent, cur_pointer.height)); 
        if let Some(node) = early_node {
            let old_node = self.nodes.replace(old_parent, node.clone()).unwrap();
            self.index_lookup.remove(&old_node);
            self.index_lookup.insert(node, old_parent);
        } 
        for index in self.bfs_nodes(cur_pointer) {  self.nodes.add_ref(index).unwrap() }
        for index in old_nodes {
            self.nodes.remove_ref(index).unwrap();
            if self.nodes.status(index).unwrap().get() == 1 && !self.is_leaf(index) {
                self.index_lookup.remove(&self.nodes.remove_ref(index).unwrap().unwrap());
            }
        }
        if early_exit { Ok(start) } else { Ok(cur_pointer) }
    }


    //Public functions used for reading
    pub fn node(&self, pointer:Index) -> Result<&Node, AccessError> {
        self.nodes.data(pointer)
    }

    pub fn child(&self, node:Index, zorder:usize) -> Result<Index, AccessError> {
        Ok( self.node(node)?.children[zorder] )
    }
    
    pub fn read(&self, start:ExternalPointer, path:&[u32]) -> Option<ExternalPointer> {
        let trail = self.get_trail(start.pointer, path);
        let Some(node_pointer) = trail.last() else { return None };
        Some(ExternalPointer::new(*node_pointer, start.height - (trail.len() as u32 - 1)))
    }

    pub fn bfs_nodes(&self, start:ExternalPointer) -> Vec<Index> {
        let mut queue = VecDeque::from([start.pointer]);
        let mut bfs_node_pointers = Vec::new();
        while let Some(node_pointer) = queue.pop_front() {
            bfs_node_pointers.push(node_pointer);
            if !self.is_leaf(node_pointer) { 
                queue.extend(self.node(node_pointer).unwrap().children) 
            }
        }
        bfs_node_pointers
    }

    pub fn get_root(&mut self, leaf:usize, height:u32) -> ExternalPointer {
        for _ in 0 .. 1 { self.nodes.add_ref(Index(leaf)).unwrap() }
        ExternalPointer::new(Index(leaf), height)
    }

}

#[derive(Serialize, Deserialize)]
struct TreeStorage {
    nodes : NodeField<Node>,
    root : ExternalPointer,
}
//Assumes constant leaf count. Eventually add more metadata
impl SparseDirectedGraph {
    pub fn save_object_json(&self, start:ExternalPointer) -> String {
        let mut data = self.bfs_nodes(start);
        data.reverse();
        let mut object_graph = Self::new(self.leaf_count);
        let root_pointer = Self::map_to(&self.nodes, &mut object_graph, &data, self.leaf_count).unwrap_or(start.pointer);
        serde_json::to_string_pretty(&TreeStorage {
            nodes : object_graph.nodes, 
            root : ExternalPointer::new(root_pointer, start.height),
        }).unwrap()
    }
    
    pub fn load_object_json(&mut self, json:String) -> ExternalPointer {
        let temp:TreeStorage = serde_json::from_str(&json).unwrap();
        let mut data = Vec::new();
        for index in 0 .. temp.nodes.internal_memory().len() {
            data.push(Index(index))
        }
        let pointer = Self::map_to(&temp.nodes, self, &data, self.leaf_count).unwrap_or(temp.root.pointer);
        ExternalPointer::new(pointer, temp.root.height)
    }

    fn map_to(source:&NodeField<Node>, to:&mut Self, data:&[Index], leaf_count:u8) -> Option<Index> {
        let mut remapped = HashMap::new();
        for i in 0 .. leaf_count as usize {
            remapped.insert(Index(i), Index(i));
        }
        for pointer in data {
            let old_node = source.data(*pointer).unwrap();
            let new_node = Node {
                children : [
                    *remapped.get(&old_node.children[0]).unwrap(),
                    *remapped.get(&old_node.children[1]).unwrap(),
                    *remapped.get(&old_node.children[2]).unwrap(),
                    *remapped.get(&old_node.children[3]).unwrap()
                ]
            };
            let new_index = to.add_node(new_node);
            remapped.insert(*pointer, new_index);
        }
        if remapped.len() != leaf_count as usize { 
            Some(*remapped.get(&data.last().unwrap()).unwrap())
        } else { None }
    }

}
