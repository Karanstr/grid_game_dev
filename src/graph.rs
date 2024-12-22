use std::collections::{HashMap, VecDeque};
use serde::{Deserialize, Serialize};
use vec_mem_heap::{MemHeap, Ownership};
use macroquad::math::{IVec2, UVec2};
pub use vec_mem_heap::{Index, AccessError};


pub struct Zorder;
#[allow(dead_code)]
impl Zorder {
    pub fn to_cell(zorder:u32, depth:u32) -> UVec2 {
        let mut cell = UVec2::ZERO;
        for layer in 0 .. depth {
            cell.x |= (zorder >> (2 * layer) & 0b1) << layer;
            cell.y |= (zorder >> (2 * layer + 1) & 0b1) << layer;
        }
        cell
    }

    pub fn from_cell(cell:UVec2, depth:u32) -> u32 {
        let mut zorder = 0;
        for layer in (0 .. depth).rev() {
            let step = (((cell.y >> layer) & 0b1) << 1 ) | ((cell.x >> layer) & 0b1);
            zorder = (zorder << 2) | step;
        }
        zorder
    }

    pub fn move_cartesianly(start_zorder:u32, depth:u32, offset:IVec2) -> Option<u32> {
        let cell = Self::to_cell(start_zorder, depth);
        let end_cell = cell.as_ivec2() + offset;
        if end_cell.min_element() < 0 || end_cell.max_element() >= 2u32.pow(depth) as i32 {
            return None
        }
        Some(Self::from_cell(UVec2::new(end_cell.x as u32, end_cell.y as u32), depth))
    }

    pub fn read(zorder:u32, layer:u32, depth:u32) -> u32 {
        zorder >> (2 * (depth - layer)) & 0b11
    }

    pub fn divergence_depth(zorder_a:u32, zorder_b:u32, depth:u32) -> Option<u32> {
        for layer in 1 ..= depth {
            if Self::read(zorder_a, layer, depth) != Self::read(zorder_b, layer, depth) {
                return Some(layer)
            }
        }
        None    
    }

    pub fn path(zorder:u32, depth:u32) -> Vec<u32> {
        let mut steps:Vec<u32> = Vec::with_capacity(depth as usize);
        for layer in 1 ..= depth {
            steps.push(Self::read(zorder, layer, depth));
        }
        steps
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

    //Private functions used for reading
    fn node(&self, index:Index) -> Result<&NodeHandler, AccessError> {
        self.nodes.data(index)
    }

    fn child(&self, node:NodePointer, zorder:usize) -> Result<NodePointer, AccessError> {
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
                        if let Err( error) = self.nodes.add_owner(child.index) {
                            self.handle_access_error(error);
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

    //Returns(zorder, depth, index)
    pub fn dfs_leaves(&self, root:NodePointer) -> Vec<(u32, u32, Index)> {
        //Arbitrary limit
        let maximum_depth = 10;
        //              zorder, depth, index
        let mut leaves = Vec::new();
        let mut stack = Vec::new();
        //       node_pointer, depth, zorder
        stack.push((root, 0u32, 0u32));

        while let Some((node, layers_deep, zorder)) = stack.pop() {
            let children = self.node(node.index).unwrap().children;
            //If we're cycling
            if children[0].index == node.index {
                leaves.push((zorder, layers_deep, children[0].index));
                continue
            }
            if layers_deep + 1 > maximum_depth { 
                println!("Graph exceeds depth limit at index {}", *node.index);
                continue
            }
            for i in (0 .. 4).rev() {
                stack.push((children[i], layers_deep + 1, (zorder << 2) | i as u32));
            }
        }
        leaves
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
            self.handle_access_error(error);
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
    pub fn save_object_json(&self, root:NodePointer) -> String {
        let mut data = self.bfs_nodes(root);
        data.reverse();
        let mut object_graph = Self::new(self.leaf_count);
        let _ = Self::map_to(&self.nodes, &mut object_graph, &data);
        serde_json::to_string_pretty(&object_graph).unwrap()
    }
    
    //Assumes leaf_count constant
    pub fn load_object_json(&mut self, json:String) -> NodePointer {
        #[derive(Deserialize)]
        struct Helper {
            nodes: MemHeap<NodeHandler>,
            //leaf_count : u8
        }
        let helper:Helper = serde_json::from_str(&json).unwrap();
        let mut data = Vec::new();
        for index in 0 .. helper.nodes.length() {
            data.push(NodePointer::new(Index(index)))
        }
        Self::map_to(&helper.nodes, self, &data)
    }

    fn map_to(source:&MemHeap<NodeHandler>, to:&mut Self, data:&[NodePointer]) -> NodePointer {
        let mut remapped = HashMap::new();
        remapped.insert(Index(0), Index(0));
        remapped.insert(Index(1), Index(1));
        remapped.insert(Index(2), Index(2));
        remapped.insert(Index(3), Index(3));
        remapped.insert(Index(4), Index(4));
        remapped.insert(Index(5), Index(5));
        remapped.insert(Index(6), Index(6));
        remapped.insert(Index(7), Index(7));
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

