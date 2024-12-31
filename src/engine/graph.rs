use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use vec_mem_heap::{MemHeap, Ownership};
pub use vec_mem_heap::{Index, AccessError};
use std::hash::Hash;

#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct Root {
    pub pointer : NodePointer,
    pub height : u32
}
impl Root {
    pub fn new(pointer:NodePointer, height:u32) -> Self {
        Self { pointer, height }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodePointer { 
    pub index : Index,
}
impl NodePointer {
    pub fn new(index:Index) -> Self {
        Self { index }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeHandler {
    pub children : [NodePointer; 4]
}
impl NodeHandler {
    pub fn new(children:[NodePointer; 4]) -> Self {
        Self { children : children }
    }
}

pub struct SparseDirectedGraph {
    nodes : MemHeap<NodeHandler>,
    index_lookup : HashMap<NodeHandler, Index>,
    pub leaf_count : u8, 
}
impl SparseDirectedGraph {
    //Utility
    pub fn new(leaf_count:u8) -> Self {
        let mut instance = Self {
            nodes : MemHeap::new(),
            index_lookup : HashMap::new(),
            leaf_count
        };
        for i in 0 .. leaf_count {
            instance.add_node(
                NodeHandler::new([NodePointer::new(Index(i as usize)); 4]),
                true
            );
        }
        instance
    }
    
    fn get_trail(&self, root:NodePointer, path:&[u32]) -> Vec<NodePointer>  {
        let mut trail = vec![root];
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
        let mut dangling = 0;
        for index in 0 .. self.nodes.length() {
            let cur_index = Index(index);
            if let Ok(status) = self.nodes.status(cur_index) {
                if let Ownership::Fine(_) = status {
                    reserved_memory += 1;
                } else { dangling += 1 }
            } else { free_memory += 1 }
        }

        println!("There are {} nodes within the tree, consisting of:", reserved_memory);
        println!("{} dangling nodes and {} free slots", dangling - self.leaf_count, free_memory);

    }

    //Private functions used for writing
    fn find_index(&self, node:&NodeHandler) -> Option<Index> {
        self.index_lookup.get(node).copied()
    }

    fn dec_owners(&mut self, index:Index) {
        let mut stack = Vec::from([index]);
        while let Some(next_index) = stack.pop() {
            match self.nodes.remove_owner(next_index) {
                Ok(Some(node)) => {
                    for child in node.children.iter() {
                        stack.push(child.index)
                    }
                    self.index_lookup.remove(&node);
                },
                Err( error ) => { self.handle_access_error(error, "Decrease Owners".to_owned()) }
                Ok(None) => {},
            }
        }
    }

    fn add_node(&mut self, node:NodeHandler, protected:bool) -> Index {
        match self.find_index(&node) {
            Some(index) => index,
            None => {
                let node_dup = node.clone();
                let index = self.nodes.push(node, protected);
                self.index_lookup.insert(node_dup, index);
                let children = self.node(index).unwrap().children;
                for child in children {
                    if child.index != index {
                        if let Err( error) = self.nodes.add_owner(child.index) {
                            self.handle_access_error(error, "Add Node".to_owned());
                        }
                    }
                }
                index
            }
        }
    }

    fn handle_access_error(&self, error:AccessError, location:String) {
        match error {
            AccessError::ProtectedMemory(index) if *index < self.leaf_count as usize => {}
            error => {
                dbg!(error, location);
            }
        }
    }

    //Public functions used for writing
    pub fn set_node(&mut self, root:NodePointer, path:&[u32], new_node:NodePointer) -> Result<NodePointer, AccessError> {
        let trail = self.get_trail(root, path);
        let mut cur_node_pointer = new_node;
        for cur_depth in (0 .. path.len()).rev() {
            let parent = if cur_depth < trail.len() { trail[cur_depth] } else { *trail.last().unwrap() };
            let parent_node_pointer =  {
                let mut new_parent = self.node(parent.index)?.clone();
                new_parent.children[path[cur_depth] as usize] = cur_node_pointer;
                new_parent
            };
            cur_node_pointer.index = self.add_node(parent_node_pointer, false);
        }
        self.swap_root(root, cur_node_pointer);
        Ok( cur_node_pointer )
    }

    //Public functions used for reading
    pub fn node(&self, index:Index) -> Result<&NodeHandler, AccessError> {
        self.nodes.data(index)
    }

    pub fn child(&self, node:NodePointer, zorder:usize) -> Result<NodePointer, AccessError> {
        Ok( self.node(node.index)?.children[zorder] )
    }

    pub fn read(&self, root:NodePointer, path:&[u32]) -> (NodePointer, u32) {
        let trail = self.get_trail(root, path);
        let Some(node_pointer) = trail.last() else { panic!("Trail is broken again") };
        (*node_pointer, trail.len() as u32 - 1)
    }

    pub fn bfs_nodes(&self, root:NodePointer) -> Vec<NodePointer> {
        let mut queue = VecDeque::from([root]);
        let mut bfs_node_pointers = Vec::new();
        while let Some(node_pointer) = queue.pop_front() {
            if let Ok(node) = self.node(node_pointer.index) {
                bfs_node_pointers.push(node_pointer);
                for child in node.children {
                    if child.index != node_pointer.index {
                        queue.push_back(child);
                    }
                }
            }
        }
        bfs_node_pointers
    }

    //Public functions used for root manipulation
    pub fn get_root(&self, index:usize) -> NodePointer {
        NodePointer {
            index : Index(index),
        }
    }

    pub fn swap_root(&mut self, old_root:NodePointer, new_root:NodePointer) {
        if let Err( error) = self.nodes.add_owner(new_root.index) {
            self.handle_access_error(error, "Swap Root".to_owned());
        }
        self.dec_owners(old_root.index);
    }

}

#[derive(Serialize, Deserialize)]
struct TreeStorage {
    nodes : MemHeap<NodeHandler>,
    root : Root,
}
//Assumes constant leaf count. Eventually add more metadata
impl SparseDirectedGraph {
    pub fn save_object_json(&self, root:Root) -> String {
        let mut data = self.bfs_nodes(root.pointer);
        data.reverse();
        let mut object_graph = Self::new(self.leaf_count);
        let root_pointer = Self::map_to(&self.nodes, &mut object_graph, &data, self.leaf_count).unwrap_or(root.pointer);
        serde_json::to_string_pretty(&TreeStorage {
            nodes : object_graph.nodes, 
            root : Root::new(root_pointer, root.height),
        }).unwrap()
    }
    
    pub fn load_object_json(&mut self, json:String) -> Root {
        let temp:TreeStorage = serde_json::from_str(&json).unwrap();
        let mut data = Vec::new();
        for index in 0 .. temp.nodes.length() {
            data.push(NodePointer::new(Index(index)))
        }
        let pointer = Self::map_to(&temp.nodes, self, &data, self.leaf_count).unwrap_or(temp.root.pointer);
        Root::new(pointer, temp.root.height)
    }

    fn map_to(source:&MemHeap<NodeHandler>, to:&mut Self, data:&[NodePointer], leaf_count:u8) -> Option<NodePointer> {
        let mut remapped = HashMap::new();
        for i in 0 .. leaf_count as usize {
            remapped.insert(Index(i), Index(i));
        }
        for pointer in data {
            let old_node = source.data(pointer.index).unwrap();
            let new_node = NodeHandler {
                children : [
                    NodePointer::new(*remapped.get(&old_node.children[0].index).unwrap()),
                    NodePointer::new(*remapped.get(&old_node.children[1].index).unwrap()),
                    NodePointer::new(*remapped.get(&old_node.children[2].index).unwrap()),
                    NodePointer::new(*remapped.get(&old_node.children[3].index).unwrap())
                ]
            };
            let new_index = to.add_node(new_node, false);
            remapped.insert(pointer.index, new_index);
        }
        if remapped.len() != leaf_count as usize { 
            Some(NodePointer::new(*remapped.get(&data.last().unwrap().index).unwrap()))
        } else { None }
    }

}
