use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
pub use vec_mem_heap::{NodeField, Index as InternalPointer, prelude::AccessError};
use std::hash::Hash;
use derive_new::new;

//Replace memheap index with a generic, making things simpler

#[derive(Debug, Copy, Clone, Serialize, Deserialize, new)]
pub struct ExternalPointer {
    pub pointer : InternalPointer,
    pub height : u32
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, new)]
pub struct Node {
    pub children : [InternalPointer; 4],
}

pub struct SparseDirectedGraph {
    nodes : NodeField<Node>,
    index_lookup : HashMap<Node, InternalPointer>,
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
            instance.add_node(Node::new([InternalPointer(i as usize); 4]));
        }
        instance
    }
    
    pub fn is_leaf(&self, index:InternalPointer) -> bool {
        *index < self.leaf_count as usize
    }

    fn get_trail(&self, start:InternalPointer, path:&[u32]) -> Vec<InternalPointer>  {
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

    /*pub fn profile(&self) {
        println!("---------- STATUS ----------");
        let mut free_memory = 0;
        let mut reserved_memory = 0;
        for index in 0 .. self.nodes.length() {
            let cur_index = InternalPointer(index);
            if let Ok(status) = self.nodes.status(cur_index) {
                if let Ownership::Fine(_) = status {
                    reserved_memory += 1;
                } else { free_memory += 1 }
            } else { free_memory += 1 }
        }

        println!("There are {} nodes within the tree, consisting of:", reserved_memory);
        println!("{} dangling nodes and {} free slots", dangling - self.leaf_count, free_memory);

    }*/

    //Private functions used for writing
    fn find_index(&self, node:&Node) -> Option<InternalPointer> {
        self.index_lookup.get(node).copied()
    }

    fn dec_owners(&mut self, pointer:InternalPointer) {
        let mut stack = Vec::from([pointer]);
        while let Some(next_index) = stack.pop() {
            if self.is_leaf(next_index) { continue; }
            match self.nodes.remove_ref(next_index) {
                Ok(Some(node)) => {
                    for child in node.children {
                        stack.push(child)
                    }
                    self.index_lookup.remove(&node);
                },
                Err( error ) => { dbg!(error, "decrease owners"); }
                Ok(None) => {},
            }
        }
    }

    fn add_node(&mut self, node:Node) -> InternalPointer {
        match self.find_index(&node) {
            Some(pointer) => pointer,
            None => {
                let pointer = self.nodes.push(node.clone());
                self.index_lookup.insert(node, pointer);
                if self.is_leaf(pointer) { return pointer }
                let children = self.node(pointer).unwrap().children;
                for child in children {
                    if let Err( error) = self.nodes.add_ref(child) {
                        dbg!(error, "add ref");
                    }
                }
                pointer
            }
        }
    }

    //Public functions used for writing
    pub fn set_node(&mut self, start:ExternalPointer, path:&[u32], new_pointer:InternalPointer) -> Result<ExternalPointer, AccessError> {
        let trail = self.get_trail(start.pointer, path);
        let mut cur_pointer = new_pointer;
        for cur_depth in (0 .. path.len()).rev() {
            let parent = if cur_depth < trail.len() { trail[cur_depth] } else { *trail.last().unwrap() };
            let parent_node_pointer =  {
                let mut new_parent = self.node(parent)?.clone();
                new_parent.children[path[cur_depth] as usize] = cur_pointer;
                new_parent
            };
            cur_pointer = self.add_node(parent_node_pointer);
        }
        self.swap_root(start.pointer, cur_pointer);
        Ok( ExternalPointer::new( cur_pointer, start.height ) )
    }

    pub fn swap_root(&mut self, old_root:InternalPointer, new_root:InternalPointer) {
        if let Err( error) = self.nodes.add_ref(new_root) {
            dbg!(error, "swap root");
        }
        if let Err( error) = self.nodes.remove_ref(old_root) {
            dbg!(error, "swap root");
        }
    }

    //Public functions used for reading
    pub fn node(&self, pointer:InternalPointer) -> Result<&Node, AccessError> {
        self.nodes.data(pointer)
    }

    pub fn child(&self, node:InternalPointer, zorder:usize) -> Result<InternalPointer, AccessError> {
        Ok( self.node(node)?.children[zorder] )
    }

    pub fn read(&self, start:ExternalPointer, path:&[u32]) -> ExternalPointer {
        let trail = self.get_trail(start.pointer, path);
        let Some(node_pointer) = trail.last() else { panic!("Trail is broken again") };
        ExternalPointer::new(*node_pointer, start.height - (trail.len() as u32 - 1))
    }

    pub fn bfs_nodes(&self, start:ExternalPointer) -> Vec<InternalPointer> {
        let mut queue = VecDeque::from([start.pointer]);
        let mut bfs_node_pointers = Vec::new();
        while let Some(node_pointer) = queue.pop_front() {
            if let Ok(node) = self.node(node_pointer) {
                bfs_node_pointers.push(node_pointer);
                for child in node.children {
                    if !self.is_leaf(child) {
                        queue.push_back(child);
                    }
                }
            }
        }
        bfs_node_pointers
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
            data.push(InternalPointer(index))
        }
        let pointer = Self::map_to(&temp.nodes, self, &data, self.leaf_count).unwrap_or(temp.root.pointer);
        ExternalPointer::new(pointer, temp.root.height)
    }

    fn map_to(source:&NodeField<Node>, to:&mut Self, data:&[InternalPointer], leaf_count:u8) -> Option<InternalPointer> {
        let mut remapped = HashMap::new();
        for i in 0 .. leaf_count as usize {
            remapped.insert(InternalPointer(i), InternalPointer(i));
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
