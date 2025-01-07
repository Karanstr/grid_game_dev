use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use vec_mem_heap::{MemHeap, Ownership};
pub use vec_mem_heap::{Index, AccessError};
use std::hash::Hash;
use derive_new::new;

#[derive(Debug, Copy, Clone, Serialize, Deserialize, new)]
pub struct ExternalPointer {
    pub pointer : InternalPointer,
    pub height : u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, new)]
pub struct InternalPointer {
    pub index : Index,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, new)]
pub struct Node {
    pub children : [InternalPointer; 4],
}


pub struct SparseDirectedGraph {
    nodes : MemHeap<Node>,
    index_lookup : HashMap<Node, InternalPointer>,
    leaf_count : u8, 
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
                Node::new([InternalPointer::new(Index(i as usize)); 4]),
                true,
            );
        }
        instance
    }
    
    pub fn is_leaf(&self, index:Index) -> bool {
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
    fn find_index(&self, node:&Node) -> Option<InternalPointer> {
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

    fn add_node(&mut self, node:Node, protected:bool) -> InternalPointer {
        match self.find_index(&node) {
            Some(pointer) => pointer,
            None => {
                let pointer = InternalPointer::new(self.nodes.push(node.clone(), protected));
                self.index_lookup.insert(node, pointer);
                let children = self.node(pointer.index).unwrap().children;
                for child in children {
                    if *child.index >= self.leaf_count as usize {
                        if let Err( error) = self.nodes.add_owner(child.index) {
                            self.handle_access_error(error, "Add Node".to_owned());
                        }
                    }
                }
                pointer
            }
        }
    }

    fn handle_access_error(&self, error:AccessError, location:String) {
        match error {
            AccessError::ProtectedMemory(index) if self.is_leaf(index) => {}
            error => {
                dbg!(error, location);
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
                let mut new_parent = self.node(parent.index)?.clone();
                new_parent.children[path[cur_depth] as usize] = cur_pointer;
                new_parent
            };
            cur_pointer = self.add_node(parent_node_pointer, false);
        }
        self.swap_root(start.pointer, cur_pointer);
        Ok( ExternalPointer::new( cur_pointer, start.height ) )
    }

    pub fn swap_root(&mut self, old_root:InternalPointer, new_root:InternalPointer) {
        if let Err( error) = self.nodes.add_owner(new_root.index) {
            self.handle_access_error(error, "Swap Root".to_owned());
        }
        self.dec_owners(old_root.index);
    }

    //Public functions used for reading
    pub fn node(&self, index:Index) -> Result<&Node, AccessError> {
        self.nodes.data(index)
    }

    pub fn child(&self, node:InternalPointer, zorder:usize) -> Result<InternalPointer, AccessError> {
        Ok( self.node(node.index)?.children[zorder] )
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
            if let Ok(node) = self.node(node_pointer.index) {
                bfs_node_pointers.push(node_pointer);
                for child in node.children {
                    if !self.is_leaf(child.index) {
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
    nodes : MemHeap<Node>,
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
        for index in 0 .. temp.nodes.length() {
            data.push(InternalPointer::new(Index(index)))
        }
        let pointer = Self::map_to(&temp.nodes, self, &data, self.leaf_count).unwrap_or(temp.root.pointer);
        ExternalPointer::new(pointer, temp.root.height)
    }

    fn map_to(source:&MemHeap<Node>, to:&mut Self, data:&[InternalPointer], leaf_count:u8) -> Option<InternalPointer> {
        let mut remapped = HashMap::new();
        for i in 0 .. leaf_count as usize {
            remapped.insert(Index(i), Index(i));
        }
        for pointer in data {
            let old_node = source.data(pointer.index).unwrap();
            let new_node = Node {
                children : [
                    InternalPointer::new(*remapped.get(&old_node.children[0].index).unwrap()),
                    InternalPointer::new(*remapped.get(&old_node.children[1].index).unwrap()),
                    InternalPointer::new(*remapped.get(&old_node.children[2].index).unwrap()),
                    InternalPointer::new(*remapped.get(&old_node.children[3].index).unwrap())
                ]
            };
            let new_index = to.add_node(new_node, false);
            remapped.insert(pointer.index, new_index.index);
        }
        if remapped.len() != leaf_count as usize { 
            Some(InternalPointer::new(*remapped.get(&data.last().unwrap().index).unwrap()))
        } else { None }
    }

}
