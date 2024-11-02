use std::collections::HashMap;
use std::hash::Hash;
//Figure out modules and libraries and all that. I've no clue how they work
use crate::fake_heap::FakeHeap; 
pub use crate::fake_heap::{Index, AccessError};

//Could be generalized to n dimensional path. I don't care atm tho
#[derive(Debug)]
pub struct Path2D {
    directions : Vec<usize>
} 

impl Path2D {
    pub fn from(bit_path:usize, steps:usize) -> Self {
        let mut directions:Vec<usize> = Vec::with_capacity(steps);
        for step in 0 .. steps {
            directions.push((bit_path >> (2 * (steps - step - 1))) & 0b11);
        }
        Self { directions }
    }
}


#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Node {
    child_indexes:Vec<Index>,
}

impl Node {

    fn new(child_count:usize) -> Self {
        Self {
            child_indexes: vec![Index(0); child_count],
        }
    }
    
    fn child(&self, child_direction:usize) -> Index {
        self.child_indexes[child_direction]
    }

}


//Generalized version of the SVDAG, removes fixed depth dimensions
pub struct DirectedGraph {
    nodes : FakeHeap<Node>,
    index_lookup : HashMap<Node, Index>,
}

impl DirectedGraph {

    pub fn new() -> Self {
        let empty_node = Node::new(4);
        let mut full_node = Node::new(4);
        for child in 0 .. 4 {
            full_node.child_indexes[child] = Index(1);
        }
        let mut instance = Self {
            nodes : FakeHeap::new(),
            index_lookup : HashMap::new()
        };
        instance.add_node(empty_node, true);
        instance.add_node(full_node, true);
        instance
    }

    pub fn empty_root(&mut self) -> Index {
        Index(0)
    }

    fn node(&self, index:Index) -> Result<&Node, AccessError> {
        self.nodes.data(index)
    }

    fn child(&self, index:Index, direction:usize) -> Result<Index, AccessError> {
        Ok(self.node(index)?.child(direction))
    }

    fn get_trail(&self, root:Index, path:&Path2D) -> Result<Vec<Index>, AccessError> {
        let mut trail:Vec<Index> = vec![root];
        for step in 0 .. path.directions.len() - 1 {
            match self.child(trail[step], path.directions[step]) {
                Ok(child_index) if child_index != trail[step] => trail.push(child_index),
                Ok(_) => break,
                Err(error) => return Err( error )
            };
        }
        Ok( trail )
    }

    pub fn read_destination(&self, root:Index, path:&Path2D) -> Result<Index, AccessError> {
        let trail = self.get_trail(root, path)?;
        match trail.last() {
            Some(index) => Ok( self.child(*index, path.directions[trail.len() - 1])? ),
            //Can't read from the end of a trail if the trail is empty
            None => Err( AccessError::InvalidRequest )
        }
    }

    fn dec_ref_count(&mut self, index:Index) {
        let mut stack:Vec<Index> = Vec::new();
        stack.push( index );
        while stack.len() != 0 {
            match self.nodes.remove_ref(stack.pop().unwrap()) {
                Ok(Some(node)) => {
                    for child in node.child_indexes.iter() {
                        stack.push(*child)
                    }
                    self.index_lookup.remove(&node);
                },
                Ok(None) | Err(AccessError::ProtectedMemory(_)) => (),
                Err( error ) => {
                    dbg!(error);
                }
            }
        }
    }

    fn find_index(&self, node:&Node) -> Option<Index> {
        self.index_lookup.get(node).copied()
    }

    fn add_node(&mut self, node:Node, protected:bool) -> Index {
        match self.find_index(&node) {
            Some(index) => index,
            None => {
                let node_dup = node.clone();
                let index = self.nodes.push(node, protected);
                self.index_lookup.insert(node_dup, index);
                let node_kids = self.node(index).unwrap().child_indexes.clone();
                for child in node_kids {
                    if child != index { //Nodes aren't allowed to keep themselves alive.
                        match self.nodes.add_ref(child) {
                            Ok(_) | Err( AccessError::ProtectedMemory(_) ) => (),
                            Err( error ) => { dbg!(error); () }
                        }
                    }
                }
                index
            }
        }
    }

    pub fn set_node_child(&mut self, root:Index, path:&Path2D, child:Index) -> Result<Index, AccessError> {
        let trail = self.get_trail(root, path)?;
        let mut new_index = child;
        let steps = path.directions.len() - 1;
        for step in 0 ..= steps {
            let mut node = if steps - step < trail.len() {
                match self.node(trail[steps - step]) {
                    Ok(node) => node.clone(),
                    Err(AccessError::FreeMemory(_)) => Node::new(4),
                    Err( error ) => return Err( error ),
                }
            } else { Node::new(4) };
            node.child_indexes[path.directions[steps - step]] = new_index;
            new_index = self.add_node(node, false);
        }
        if let Err( error ) = self.nodes.add_ref(new_index) { dbg!(error); }
        self.dec_ref_count(root);
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

