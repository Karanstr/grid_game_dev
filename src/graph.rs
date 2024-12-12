use std::collections::HashMap;
use vec_mem_heap::{MemHeap, Ownership};
pub use vec_mem_heap::{Index, AccessError};


mod node_stuff {
    use super::Index;
    use std::hash::Hash;


    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
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
    
    #[derive(Clone, Debug, PartialEq, Eq, Hash)]
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
pub struct SparseDirectedGraph {
    nodes : MemHeap<NodeHandler>,
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

    //Private functions used for reading
    fn node(&self, index:Index) -> Result<&NodeHandler, AccessError> {
        self.nodes.data(index)
    }

    fn child(&self, node:NodePointer, zorder:usize) -> Result<NodePointer, AccessError> {
        Ok( self.node(node.index)?.children[zorder] )
    }

    //Add cyclicity here
    fn get_trail(&self, root:NodePointer, path:&Vec<u32>) -> Vec<NodePointer>  {
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
                Err( error ) => { self.handle_access_error(error) }
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
                        match self.nodes.add_owner(child.index) {
                            Ok(_) => (),
                            Err( error ) => { self.handle_access_error(error); }
                        }
                    }
                }
                index
            }
        }
    }

    fn handle_access_error(&self, error:AccessError) {
        match error {
            AccessError::ProtectedMemory(index) if *index < self.leaf_count as usize => {}
            error => {
                dbg!(error);
            }
        }
    }

    //Public functions used for writing
    //Add cyclicity here.
    pub fn set_node(&mut self, root:NodePointer, path:&Vec<u32>, new_node:NodePointer) -> Result<NodePointer, AccessError> {
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
        if let Err( error ) = self.nodes.add_owner(cur_node_pointer.index) { self.handle_access_error(error) }
        self.dec_owners(root.index);
        Ok ( cur_node_pointer )
    }


    //Public functions used for reading
    pub fn read(&self, root:NodePointer, path:&Vec<u32>) -> (NodePointer, u32) {
        let trail = self.get_trail(root, path);
        if let Some(node_pointer) = trail.last() {
            (*node_pointer, trail.len() as u32 - 1)
        } else { panic!("Trail is broken again") }
    }

    pub fn dfs_leaves(&self, root:NodePointer) -> Vec<(u32, u32, Index)> {
        //Arbitrary limit
        let maximum_render_depth = 10;
        //              zorder, depth, index
        let mut leaves = Vec::new();
        let mut stack = Vec::new();
        //       node_pointer, depth, zorder
        stack.push((root, 0u32, 0u32));

        while stack.len() != 0 {
            let (node, layers_deep, zorder) = stack.pop().unwrap();
            let children = self.node(node.index).unwrap().children;
            //If we're cycling
            if children[0].index == node.index {
                leaves.push((zorder, layers_deep, children[0].index));
                continue
            }
            if layers_deep + 1 > maximum_render_depth { 
                println!("Graph exceeds depth limit at index {}", *node.index);
                continue
            }
            for i in 0 .. 4 {
                stack.push((children[i], layers_deep + 1, (zorder << 2) | i as u32));
            }
        }
        leaves
    }

    pub fn find_corners(&self, root:NodePointer) {
        
    }

    //Public functions used for root manipulation
    pub fn get_root(&self, index:usize) -> NodePointer {
        NodePointer {
            index : Index(index),
        }
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

}
