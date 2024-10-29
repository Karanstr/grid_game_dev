use derive_more::{Deref, Index as Indexable, IndexMut as IndexableMut};
use macroquad::math::*;
//Figure out modules and libraries and all that. I've no clue how they work
use crate::garbagetracker::{ReferenceTracker, ReferenceStatus}; 

#[derive(Debug)]
pub struct Node {
    pub child_indexes:Vec<usize>,
    pub gc:ReferenceTracker,
}

impl PartialEq for Node {
    fn eq(&self, other:&Self) -> bool {
        self.child_indexes == other.child_indexes
    }
}

impl Node {
    fn new(child_count:usize, protected:bool) -> Self {
        Self {
            child_indexes: vec![0; child_count],
            //We have to garbage collect ourselves :(
            gc: ReferenceTracker::new(protected), 
        }
    }

    fn clone_without_ref(&self, protected:bool) -> Self {
        Self {
            child_indexes: self.child_indexes.clone(),
            gc: ReferenceTracker::new(protected),
        }
    }
    
    fn read_child(&self, child_direction:usize) -> Index {
        Index(self.child_indexes[child_direction])
    }


}

//Stores dimension for (currently not implemented error prevention)
#[derive(Debug)]
pub struct Path {
    _dimension : usize,
    directions : Vec<usize>
} 

impl Path {
    pub fn from(bit_path:usize, steps:usize, dimension:usize) -> Self {
        let mut directions:Vec<usize> = Vec::with_capacity(steps);
        let mut mask = 0; for _ in 0 .. dimension { mask = (mask << 1) | 1 }
        for step in 0 .. steps {
            directions.push((bit_path >> (dimension * (steps - step - 1))) & mask);
        }
        Self {
            _dimension : dimension,
            directions
        }
    }
}

#[derive(Deref, PartialEq, PartialOrd, Clone, Copy)]
pub struct Index(pub usize);

//For now only supports a graph where each node has four children.
//This is incredibly arbitrary and incredibly easy to change
#[derive(Indexable, IndexableMut)]
pub struct DirectedGraph(Vec<Node>);

impl DirectedGraph {

    pub fn new() -> Self {
        let empty_node = Node::new(4, true);
        let mut full_node = Node::new(4, true);
        for child in 0 .. 4 {
            full_node.child_indexes[child] = 1;
        }
        Self(vec![empty_node, full_node])
    }

    pub fn get_empty_root(&mut self) -> Index {
        Index(0)
    }


    //Private methods used to read from the graph
    fn get_mut_node(&mut self, index:Index) -> Option<&mut Node> {
        //If the index is out of range or in a protected slot
        if index > self.last_index() || self[*index].gc.get_status() == ReferenceStatus::Protected {
            None
        } else {
            Some(&mut self[*index])
        }
    }

    fn read_child(&self, index:Index, child_direction:usize) -> Option<Index> {
        Some(self.get_node(index)?.read_child(child_direction))
    }

    fn find_node(&self, node:&Node, ignore_protected:bool) -> Option<Index> {
        //Eventually make this automatic, determining an automatic allocation for protected nodes?
        let start_index = if ignore_protected { 2 } else { 0 };
        for cur_index in start_index ..= *self.last_index() {
            let index = Index(cur_index);
            let cur_node = self.get_node(index)?;
            if node == cur_node {
                return Some(index);
            }
        }
        None
    }

    fn get_or_make_empty_index(&mut self) -> Index {
        let empty_node = Node::new(4, false);
        match self.find_node(&empty_node, true) {
            Some(index) => index,
            _ => {
                self.0.push(empty_node);
                self.last_index()
            }
        }
    }

    fn get_trail(&self, root:Index, path:&Path) -> Vec<Index> {
        let mut trail:Vec<Index> = vec![root];
        for step in 0 .. path.directions.len() - 1 {
            match self.read_child(trail[step], path.directions[step]) {
                Some(child_index) if child_index != trail[step] => trail.push(child_index),
                _ => break
            };
        }
        trail
    }

    pub fn last_index(&self) -> Index {
        Index(self.0.len() - 1)
    }


    //Public methods used to read from the graph
    pub fn get_node(&self, index:Index) -> Option<&Node> {
        if index > self.last_index() { 
            None 
        } else {
            Some(&self[*index])
        }
    }

    pub fn read_destination(&self, root:Index, path:&Path) -> Option<Index> {
        let trail = self.get_trail(root, path);
        self.read_child(*trail.last()?, path.directions[trail.len() - 1])
    }


    //Private methods used to modify graph data
    fn free_node(&mut self, index:Index) {
        self[*index] = Node::new(4, false);
    }

    fn dec_ref_count(&mut self, index:Index) {
        let mut stack:Vec<Index> = Vec::new();
        stack.push( index );
        while stack.len() != 0 {
            let cur_index = stack.pop().unwrap();
            if let Some(node) = self.get_mut_node(cur_index) {
                match node.gc.modify_ref(-1) {
                    Ok(status) => if let ReferenceStatus::Zero = status {
                        for child_direction in 0 .. node.child_indexes.len() {
                            stack.push( node.read_child(child_direction) );
                        }
                        self.free_node(cur_index);
                    },
                    Err(message) => eprintln!("{message}")
                }
            }
        }
    }

    fn inc_ref_count(&mut self, index:Index) {
        match self.get_mut_node(index) {
            Some(node) => {_ = node.gc.modify_ref(1)},
            None => ()
        };
    }

    fn add_node(&mut self, node:Node) -> Index {
        match self.find_node(&node, false) {
            Some(index) => index,
            None => {
                let index = self.get_or_make_empty_index();
                for child_direction in 0 .. node.child_indexes.len() {
                    self.inc_ref_count(node.read_child(child_direction));
                }
                self[*index] = node;
                index
            }
        }
    }

    fn transfer_reference(&mut self, giver:Index, reciever:Index) {
        self.inc_ref_count(reciever);
        self.dec_ref_count(giver);
    }


    //Public methods used to modify graph data
    pub fn set_node_child(&mut self, root:Index, path:&Path, child:Index) -> Index {
        let trail = self.get_trail(root, path);
        let mut new_index = child;
        let steps = path.directions.len() - 1;
        for step in 0 ..= steps {
            let mut node = if steps - step < trail.len() {
                match self.get_node(trail[steps - step]) {
                    Some(node) => node.clone_without_ref(false),
                    None => Node::new(4, false)
                }
            } else { Node::new(4, false) };
            node.child_indexes[path.directions[steps - step]] = *new_index;
            new_index = self.add_node(node);
        }
        self.transfer_reference(root, new_index);

        new_index
    }


}
   
    /* 
    Not 2d/dimensionless yet. This is the next step.
     - Figure out where to put them, they don't feel like they belong in such a generalized graph.
     - Make them functional with the new graph layout. Should be simple..?
     - Generalize them to n dimensions, along with the rest of the graph.
    */

    /*
    fn _compact_root_children(&mut self, root:&mut NodeAddress) -> bool {
        let child_directions = [1, 0];
        let mut new_root_node = Node::new_empty();
        let children = self.get_node(&root).child_indexes;
        for index in 0 .. child_directions.len() {
            let address = NodeAddress::new(root.layer - 1, children[index]);
            let node = self.get_node(&address);
            let (child_count, last_index) = node.count_kids_and_get_last();
            if child_count > 1 || last_index != child_directions[index] {
                return false //Cannot compact root
            }
            new_root_node.child_indexes[index] = node.child_indexes[child_directions[index]];
        } //If we don't terminate we are safe to lower the root
        let new_root_index = self.add_node(root.layer - 1, new_root_node);
        self.transfer_reference(&root, &NodeAddress::new(root.layer - 1, new_root_index));
        root.layer -= 1;
        root.index = new_root_index;
        true //Successfully compacted root
    }

    //This has a problem with my current system. Hmm
    //Only works with 2 dimensions, up to root level 5 (64x64 cell area)
    pub fn df_to_bin_grid(&self, root:Index, root_layer:usize) -> Vec<u64> {
        let blocks_per_side = 2usize.pow(1 + root_layer as u32);
        let mut bin_grid:Vec<u64> = Vec::new();
        bin_grid.resize(blocks_per_side, 0);

        //Storing indexes their z-order cells, and the current layer (as determined by the root_layer)
        let mut stack: Vec<(Index, u32)> = Vec::new();
        stack.push( (root.clone(), 0) );

        //Figure out how to make this thread-compatible soon
        while stack.len() > 0 {
            let (cur_address, cur_z_order) = stack.pop().unwrap();
            //Should be safe to unwarp here.
            let cur_node = match self.get_node(&cur_address) {
                Some(node) => node,
                None => continue
            };

            for child in 0 .. cur_node.child_indexes.len() {
                let child_index = cur_node.child_indexes[child];
                let kid_z_order = (cur_z_order << 2) | child as u32;
                if cur_node.is_child_leaf(child) {
                    if child_index == 0 { continue }
                    let cartesian = zorder_to_cartesian(kid_z_order, root.layer);
                    bin_grid[cartesian.y as usize] |= 1 << cartesian.x;
                } else {
                    stack.push( (NodeAddress::new(cur_address.layer - 1, child_index), kid_z_order) );
                }
            }
        }
        bin_grid
    } 


    pub fn zorder_to_cartesian(mut cell:u32, root_layer:usize) -> UVec2 {
        let mut xy = UVec2::new(0, 0);
        for layer in 0 ..= root_layer {
            xy.x |= (cell & 0b1) << layer;
            cell >>= 1;
            xy.y |= (cell & 0b1) << layer;
            cell >>= 1;
        }
        xy
    }

    */

