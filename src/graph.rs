use derive_more::Deref;
use macroquad::math::*;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
//Figure out modules and libraries and all that. I've no clue how they work
use crate::garbagetracker::{ReferenceStatus, ReferenceTracker}; 


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


//Idk why the usizes are complaining but I really don't care
#[allow(dead_code)] 
#[derive(Debug)]
pub enum AccessError {
    OutOfBoundsMemory(usize),
    ImmutableMemory(usize),
    FreeMemory(usize),
    UnspecifiedMemory
}


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
impl Eq for Node {}
impl Hash for Node {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.child_indexes.hash(state);
    }
}

impl Node {
    fn new(child_count:usize, immutable:bool) -> Self {
        Self {
            child_indexes: vec![0; child_count],
            //We have to garbage collect ourselves :(
            gc: ReferenceTracker::new(immutable), 
        }
    }

    fn clone_without_ref(&self, immutable:bool) -> Self {
        Self {
            child_indexes: self.child_indexes.clone(),
            gc: ReferenceTracker::new(immutable),
        }
    }
    
    fn read_child(&self, child_direction:usize) -> Index {
        Index(self.child_indexes[child_direction])
    }


}


#[derive(Deref, PartialEq, PartialOrd, Clone, Copy)]
pub struct Index(pub usize);

//For now only supports a directed graph where each node points to 4 other nodes (4 children).
//This is incredibly arbitrary and incredibly easy to change
//I'm not quite sure what happens if you create a multi-node-cycle (something more than a node referencing itself) though
pub struct DirectedGraph {
    nodes : Vec<Option<Node>>,
    index_lookup : HashMap<Node, Index>,
    free_indexes : Vec<Index>
}

impl DirectedGraph {

    pub fn new() -> Self {
        let empty_node = Node::new(4, true);
        let mut full_node = Node::new(4, true);
        for child in 0 .. 4 {
            full_node.child_indexes[child] = 1;
        }
        Self {
            nodes : vec![
                Some(empty_node.clone_without_ref(true)), 
                Some(full_node.clone_without_ref(true))
            ],
            index_lookup : HashMap::from([
                (empty_node, Index(0)),
                (full_node, Index(1))
            ]),
            free_indexes : Vec::new()
        }
    }

    pub fn get_empty_root(&mut self) -> Index {
        Index(0)
    }

    //Private methods used to read from the graph
    fn get_mut_node(&mut self, index:Index) -> Result<&mut Node, AccessError> {
        if index > self.last_index() { 
            Err( AccessError::OutOfBoundsMemory(*index) )
        } else {
            match &mut self.nodes[*index] {
                Some(node) if node.gc.get_status() == ReferenceStatus::Immutable => {
                    Err( AccessError::ImmutableMemory(*index) )
                },
                None => Err( AccessError::FreeMemory(*index) ),
                Some(node) => Ok(node),
            }
        }
    }

    fn read_child(&self, index:Index, child_direction:usize) -> Result<Index, AccessError> {
        Ok( self.get_node(index)?.read_child(child_direction) )
    }

    fn get_node_index(&self, node:Option<&Node>) -> Option<Index> {
        self.index_lookup.get(node?).copied()
    }

    fn get_trail(&self, root:Index, path:&Path) -> Result<Vec<Index>, AccessError> {
        let mut trail:Vec<Index> = vec![root];
        for step in 0 .. path.directions.len() - 1 {
            match self.read_child(trail[step], path.directions[step]) {
                Ok(child_index) if child_index != trail[step] => trail.push(child_index),
                Ok(_) => break,
                Err(error) => return Err( error )
            };
        }
        Ok( trail )
    }

    pub fn last_index(&self) -> Index {
        Index(self.nodes.len() - 1)
    }


    //Public methods used to read from the graph
    pub fn get_node(&self, index:Index) -> Result<&Node, AccessError> {
        if index > self.last_index() { 
            Err( AccessError::OutOfBoundsMemory(*index) ) 
        } else {
            match &self.nodes[*index] {
                None => Err( AccessError::FreeMemory(*index) ),
                Some(node) => Ok(node),
            }
        }
    }

    pub fn read_destination(&self, root:Index, path:&Path) -> Result<Index, AccessError> {
        let trail = self.get_trail(root, path)?;
        match trail.last() {
            Some(index) => Ok( self.read_child(*index, path.directions[trail.len() - 1])? ),
            //Can't read from the end of a trail if the trail is empty
            None => Err( AccessError::UnspecifiedMemory )
        }
    }


    //Private methods used to modify graph data
    fn inc_ref_count(&mut self, index:Index) -> Result<(), AccessError> {
        _ = self.get_mut_node(index)?.gc.modify_ref(1);
        Ok(())
    }

    fn dec_ref_count(&mut self, index:Index) -> Result<(), AccessError> {
        let mut stack:Vec<Index> = Vec::new();
        stack.push( index );
        while stack.len() != 0 {
            let cur_index = stack.pop().unwrap();
            let node = self.get_mut_node(cur_index)?;
            if node.gc.get_status() != ReferenceStatus::Immutable {
                match node.gc.modify_ref(-1) {
                    Ok(status) => if let ReferenceStatus::Zero = status {
                        for child_direction in 0 .. node.child_indexes.len() {
                            stack.push( node.read_child(child_direction) );
                        }
                        self.free_index(cur_index);
                    },
                    //This is an error I'm willing to let fail quietly for now. 
                    Err(message) => eprintln!("{message}")
                }
            }
        }
        Ok(())
    }

    fn transfer_reference(&mut self, giver:Index, reciever:Index) -> (Result<(), AccessError>, Result<(), AccessError>) {
        (self.inc_ref_count(reciever), self.dec_ref_count(giver))
    }

    fn free_index(&mut self, index:Index) -> Option<Node> {
        //Accessing as mutable because we're going to delete it
        //Any checks against borrowing nodes mutably should be applied here too.
        let old_node = self.get_mut_node(index)
            .expect("Attempted to free non-reserved memory.")
            .clone_without_ref(false);
        self.index_lookup.remove(&old_node);
        self.nodes[*index] = None;
        self.free_indexes.push(index);
        Some(old_node)
    }

    fn reserve_index(&mut self) -> Index {
        match self.free_indexes.pop() {
            Some(index) => index,
            None => {
                self.nodes.push(None);
                self.last_index()
            }
        }        
    }

    fn add_node(&mut self, node:Node) -> Index {
        match self.get_node_index(Some(&node)) {
            Some(index) => index,
            None => {
                let index = self.reserve_index();
                for child_direction in 0 .. node.child_indexes.len() {
                    match self.inc_ref_count(node.read_child(child_direction)) {
                        Ok(_) => (),
                        //Allowed to fail quietly, this error is caught on another level and prevented
                        Err( AccessError::ImmutableMemory(_) ) => (),
                        //This shouldn't trigger. If it does, something is wrong.
                        Err( error ) => { dbg!(error); () }
                    }
                }
                self.index_lookup.insert(node.clone_without_ref(true), index);
                self.nodes[*index] = Some(node);
                index
            }
        }
    }


    //Public methods used to modify graph data
    pub fn set_node_child(&mut self, root:Index, path:&Path, child:Index) -> Result<Index, AccessError> {
        let trail = self.get_trail(root, path)?;
        let mut new_index = child;
        let steps = path.directions.len() - 1;
        for step in 0 ..= steps {
            let mut node = if steps - step < trail.len() {
                match self.get_node(trail[steps - step]) {
                    Ok(node) => node.clone_without_ref(false),
                    Err(AccessError::FreeMemory(_)) => Node::new(4, false),
                    Err( error ) => return Err( error ),
                }
            } else { Node::new(4, false) };
            node.child_indexes[path.directions[steps - step]] = *new_index;
            new_index = self.add_node(node);
        }
        //Bunch of error checking here. We probably don't care, but can't be too safe during dev.
        let (inc_status, dec_status) = self.transfer_reference(root, new_index);
        //We ignore attempted access to ImmuntableMemory because the root may be 
        match inc_status {
            Ok(_) => (),
            Err( AccessError::ImmutableMemory(_) ) => (),
            Err( error ) => { dbg!(error); () },
        }
        match dec_status {
            Ok(_) => (),
            Err( AccessError::ImmutableMemory(_) ) => (),
            Err( error ) => { dbg!(error); () },
        }
        Ok( new_index )
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

