use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use vec_mem_heap::{MemHeap, Ownership};
pub use vec_mem_heap::{Index, AccessError};

#[derive(Copy, Clone)]
pub struct Root {
    pub pointer : NodePointer,
    pub height : u32
}
impl Root {
    pub fn new(pointer:NodePointer, height:u32) -> Self {
        Self { pointer, height }
    }
}

mod node_stuff {
    use super::Index;
    use serde::{Serialize, Deserialize};
    use std::hash::Hash;


    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct NodePointer { 
        pub index : Index,
    }
    impl NodePointer {
        pub fn new(index:Index) -> Self {
            Self {
                index, 
            }
        }
    }
    
    #[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
    pub struct NodeHandler {
        pub children : [NodePointer; 4]
    }
    

    //Re-add this compression stuff once I've seperated geometry
    /*
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Four {
        pub children : [NodePointer; 4],
    }
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Three {
        pub children : [NodePointer; 3],
    }
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Two {
        pub children : [NodePointer; 2],
    }
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct One {
        pub child : NodePointer,
    }
    impl One {
        pub fn new(index:Index) -> Self {
            Self { 
                child : NodePointer::new(index, 0b0000) 
            }
        }
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum NodeHandler {
        FourNode(Four),
        ThreeNode(Three),
        TwoNode(Two),  
        OneNode(One),
    }

    impl NodeHandler {

        pub fn child_freq(freq_mask:u8, child_zorder:usize) -> usize {
            (freq_mask as usize >> (2 * child_zorder)) & 0b11
        }
 
        pub fn raw_node(&self) -> Vec<NodePointer> {
            match self {
                Self::FourNode(node) => Vec::from(node.children),
                Self::ThreeNode(node) => Vec::from(node.children),
                Self::TwoNode(node) => Vec::from(node.children),
                Self::OneNode(node) => vec![node.child],
            }
        }

        fn read_node(&self, freq_mask:u8) -> [NodePointer; 4] {
            let children = self.raw_node();
            [
                children[Self::child_freq(freq_mask, 0)],
                children[Self::child_freq(freq_mask, 1)],
                children[Self::child_freq(freq_mask, 2)],
                children[Self::child_freq(freq_mask, 3)]
            ] 
        }

        pub fn child(&self, freq_mask:u8, child_zorder:usize) -> NodePointer {
            self.raw_node()[Self::child_freq(freq_mask, child_zorder)]
        }

        pub fn with_different_child(&self, freq_mask:u8, child_zorder:usize, child:NodePointer) -> (Self, u8) {
            let mut node = self.read_node(freq_mask);
            node[child_zorder] = child;
            Self::compress_node(node)
        }

        //Fix this
        fn compress_node( node:[NodePointer; 4]) -> (Self, u8) {
            let count:HashMap<NodePointer, u8> = HashMap::new();
            let mut freq_mask = 0;
            for i in 0 .. 4 {
                match count.get_mut(&node[i]) {
                    Some(number) => *number += 1,
                    None => _ = count.insert(node[i], 1)
                }
            }
            for 
            let mut compressed_node = Vec::new();
            ( match compressed_node.len() {
                4 => Self::FourNode(Four{
                    children : 
                } ), 
                3 => Self::ThreeNode(Three{
                    children : 
                } ), 
                2 => Self::TwoNode(Two{
                    children : 
                } ), 
                1 => Self::OneNode(One{
                    child : compressed_node[0]
                } ),
                _ => panic!("What?")
            }, 
            freq_mask)
        }

    }

    */
}

pub use node_stuff::{NodeHandler, NodePointer};
#[derive(Serialize)]
pub struct SparseDirectedGraph {
    nodes : MemHeap<NodeHandler>,
    #[serde(skip)]
    index_lookup : HashMap<NodeHandler, Index>,
    pub leaf_count : u8, 
}
impl SparseDirectedGraph {
    pub fn new(leaf_count:u8) -> Self {
        let mut instance = Self {
            nodes : MemHeap::new(),
            index_lookup : HashMap::new(),
            leaf_count
        };
        for i in 0 .. leaf_count {
            instance.add_node(
                NodeHandler{
                    children : [NodePointer::new(Index(i as usize)); 4]
                }, 
                true
            );
        }
        instance
    }

    //Functions used for reading
    pub fn node(&self, index:Index) -> Result<&NodeHandler, AccessError> {
        self.nodes.data(index)
    }

    pub fn child(&self, node:NodePointer, zorder:usize) -> Result<NodePointer, AccessError> {
        Ok( self.node(node.index)?.children[zorder] )
    }

    //Add cyclicity here
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


    //Private functions used for writing
    fn find_index(&self, node:&NodeHandler) -> Option<Index> {
        self.index_lookup.get(node).copied()
    }

    fn dec_owners(&mut self, index:Index) {
        let mut stack = Vec::new();
        stack.push( index );
        while stack.len() != 0 {
            match self.nodes.remove_owner(stack.pop().unwrap()) {
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
    //Add cyclicity here.
    pub fn set_node(&mut self, root:NodePointer, path:&[u32], new_node:NodePointer) -> Result<NodePointer, AccessError> {
        let trail = self.get_trail(root, path);
        let mut cur_node_pointer = new_node;
        let depth = path.len();
        for layer in 1 ..= depth {
            let cur_depth = depth - layer;
            let parent = if cur_depth < trail.len() { trail[cur_depth] } else { *trail.last().unwrap() };
            let parent_node_pointer =  {
                let mut new_parent = self.node(parent.index)?.clone();
                new_parent.children[path[cur_depth] as usize] = cur_node_pointer;
                new_parent
            };
            cur_node_pointer.index = self.add_node(parent_node_pointer, false);
        }
        self.swap_root(root, cur_node_pointer);
        // if let Err( error ) = self.nodes.add_owner(cur_node_pointer.index) { self.handle_access_error(error) }
        // self.dec_owners(root.index);
        Ok ( cur_node_pointer )
    }

    //Public functions used for reading
    pub fn read(&self, root:NodePointer, path:&[u32]) -> (NodePointer, u32) {
        let trail = self.get_trail(root, path);
        if let Some(node_pointer) = trail.last() {
            (*node_pointer, trail.len() as u32 - 1)
        } else { panic!("Trail is broken again") }
    }

    fn bfs_nodes(&self, root:NodePointer) -> Vec<NodePointer> {
        let mut queue = VecDeque::new();
        let mut bfs_node_pointers = Vec::new();
        queue.push_back(root);
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

    //This is a stupid, temporary function
    pub fn swap_root(&mut self, old_root:NodePointer, new_root:NodePointer) {
        if let Err( error) = self.nodes.add_owner(new_root.index) {
            self.handle_access_error(error, "Swap Root".to_owned());
        }
        self.dec_owners(old_root.index);
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

    //Create object which can be pulled into/out of the dag, where we handle all sorts of stuff.
    pub fn save_object_json(&self, root:Root) -> String {
        let mut data = self.bfs_nodes(root.pointer);
        data.reverse();
        let mut object_graph = Self::new(self.leaf_count);
        let _ = Self::map_to(&self.nodes, &mut object_graph, &data, self.leaf_count);
        #[derive(Serialize)]
        struct Helper {
            nodes : MemHeap<NodeHandler>,
            root_height : u32
        }
        serde_json::to_string_pretty(&Helper { 
            nodes : object_graph.nodes, 
            root_height : root.height 
        }).unwrap()
    }
    
    //Assumes leaf_count constant
    pub fn load_object_json(&mut self, json:String) -> Root {
        #[derive(Deserialize)]
        struct Helper {
            nodes: MemHeap<NodeHandler>,
            root_height : u32
            //leaf_count : u8
        }
        let helper:Helper = serde_json::from_str(&json).unwrap();
        let mut data = Vec::new();
        for index in 0 .. helper.nodes.length() {
            data.push(NodePointer::new(Index(index)))
        }
        let pointer = Self::map_to(&helper.nodes, self, &data, self.leaf_count);
        Root::new(pointer, helper.root_height)
    }

    fn map_to(source:&MemHeap<NodeHandler>, to:&mut Self, data:&[NodePointer], leaf_count:u8) -> NodePointer {
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
        NodePointer::new(*remapped.get(&data.last().unwrap().index).unwrap())
    }


}

