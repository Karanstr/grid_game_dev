use std::collections::HashMap;
use std::hash::Hash;

use vec_mem_heap::MemHeap;
pub use vec_mem_heap::{Index, AccessError};


// enum NodeTypes {
//     Full(Index, Index, Index, Index),
//     Three(Index, Index, Index),
//     Half(Index, Index),
//     One(Index),
//     Leaf,
//     Empty //Maybe this?
// }


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

//Assumes 2 dimensions for now
#[derive(PartialEq, Eq, Hash, Clone)]
pub struct Node {
    child_indexes: [Index; 4],
}

impl Node {

    fn new(initial_value:Index) -> Self {
        Self {
            child_indexes: [initial_value; 4],
        }
    }
    
    //Trusts direction < 4. Will panic if it does
    fn child(&self, direction:usize) -> Index {
        self.child_indexes[direction]
    }

}

#[derive(Debug)]
pub struct Location {
    pub index:Index,
    pub depth:usize,
}

//Improvement on the SDAG structure
pub struct SparsePixelDirectedGraph {
    nodes : MemHeap<Node>,
    index_lookup : HashMap<Node, Index>,
}

impl SparsePixelDirectedGraph {

    pub fn new() -> Self {
        let empty_node = Node::new(Index(0));
        let full_node = Node::new(Index(1));
        let mut instance = Self {
            nodes : MemHeap::new(),
            index_lookup : HashMap::new()
        };
        instance.add_node(empty_node, true);
        instance.add_node(full_node, true);
        instance
    }


    //Private functions used for reading
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


    //Private functions used for writing
    fn find_index(&self, node:&Node) -> Option<Index> {
        self.index_lookup.get(node).copied()
    }

    fn dec_owners(&mut self, index:Index) {
        let mut stack:Vec<Index> = Vec::new();
        stack.push( index );
        while stack.len() != 0 {
            match self.nodes.remove_owner(stack.pop().unwrap()) {
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
                        match self.nodes.add_owner(child) {
                            Ok(_) | Err( AccessError::ProtectedMemory(_) ) => (),
                            Err( error ) => { dbg!(error); () }
                        }
                    }
                }
                index
            }
        }
    }


    //Public functions used for writing
    pub fn set_node_child(&mut self, root:Index, path:&Path2D, child:Index) -> Result<Index, AccessError> {
        let trail = self.get_trail(root, path)?;
        let mut new_index = child;
        let steps = path.directions.len() - 1;
        for step in 0 ..= steps {
            let mut node = if steps - step < trail.len() {
                match self.node(trail[steps - step]) {
                    Ok(node) => node.clone(),
                    Err(AccessError::FreeMemory(_)) => Node::new(Index(0)),
                    Err( error ) => return Err( error ),
                }
            } else { Node::new(Index(0)) };
            node.child_indexes[path.directions[steps - step]] = new_index;
            new_index = self.add_node(node, false);
        }
        if let Err( error ) = self.nodes.add_owner(new_index) { dbg!(error); }
        self.dec_owners(root);
        Ok( new_index )
    }


    //Public functions used for reading
    pub fn read_destination(&self, root:Index, path:&Path2D) -> Result<Location, AccessError> {
        let trail = self.get_trail(root, path)?;
        match trail.last() {
            Some(index) => Ok( 
                Location {
                    index : self.child(*index, path.directions[trail.len() - 1])?,
                    depth : trail.len() - 1
                }
            ),
            //Can't read from the end of a trail if the trail is empty
            None => Err( AccessError::InvalidRequest )
        }
    }

    pub fn dfs_leaves(&self, root:Index) -> Vec<(u32, u32, Index)> {
        let mut leaves = Vec::new();
        let mut stack = Vec::new();
        stack.push((root, 0u32, 0u32));

        while stack.len() != 0 {
            let (cur_index, layers_deep, zorder) = stack.pop().unwrap();
            if layers_deep == 10 { //Arbitrary depth catcher to prevent infinite diving
                dbg!(*cur_index);
                continue;
            }
            //Because we're just following pointers this only fails if the structure has failed.
            let cur_node = self.node(cur_index).unwrap();
            for direction in 0 .. cur_node.child_indexes.len() {
                let child_index = cur_node.child(direction);
                if child_index == cur_index {
                    leaves.push((zorder, layers_deep, child_index));
                    //This may not generalize, but for now if it's a leaf it's a full leaf
                    break
                } else {
                    stack.push((child_index, layers_deep + 1, (zorder << 2) | direction as u32))
                }
            }
        }
        leaves
    }

    //Public functions used for root manipulation
    pub fn empty_root(&self) -> Index {
        Index(0)
    }

    pub fn raise_root_domain(&mut self, root:Index, path:&Path2D) -> Result<Index, AccessError> {
        let new_root = self.set_node_child(self.empty_root(), path, root)?;
        self.nodes.add_owner(new_root)?;
        self.nodes.remove_owner(root)?;
        Ok(new_root)
    }

    pub fn lower_root_domain(&mut self, root:Index, path:&Path2D) -> Result<Index, AccessError> {
        let new_root = self.read_destination(root, path)?.index;
        self.nodes.add_owner(new_root)?;
        self.nodes.remove_owner(root)?;
        Ok(new_root)
    }


}
   
    /* 
    Not 2d/dimensionless yet. This is the next step.
     - Figure out where to put them, they don't feel like they belong in such a generalized graph.
     - Make them functional with the new graph layout. Should be simple..?
     - Generalize them to n dimensions, along with the rest of the graph.
    */

    /*
    //This has a problem with my current system. Hmm
    //Only works with 2 dimensions, up to root level 5 (64x64 cell area)
    pub fn df_to_bin_grid(&self, root:Index, root_layer:usize) -> Vec<u64> {
        let blocks_per_side = 2usize.pow(1 + root_layer as u32);
        let mut bin_grid:Vec<u64> = Vec::new();
        bin_grid.resize(blocks_per_side, 0);

        //Storing indexes, their z-order cells, and the current layer (as determined by the root_layer)
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

