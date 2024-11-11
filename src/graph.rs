use std::collections::HashMap;
// use std::hash::Hash;

use vec_mem_heap::MemHeap;
pub use vec_mem_heap::{Index, AccessError};

#[derive(Clone, Copy, Debug, PartialEq)]
struct Child {
    index:Index,
    pre_cycles : u8
}

impl Child {
    fn new(index:Index, pre_cycles:u8) -> Self {
        Self {
            index, 
            pre_cycles
        }
    }
}

mod node_stuff {
    use super::Child;
    use std::convert::TryInto;

    //We distinguish the structs from the enum vecs because I want data stored contigously (which requires known size arrays instead of Vecs).
    //The structs will be stored in memory, and the enum below will be used to manipulate them by wrapping the structs so they can be passed around

    //Leaves should always have a config (child_mask) of 0b1111 (7)
    struct Leaf {
        children : [Child; 1],
        configs : [u8; 1]
    }
    struct Full {
        children : [Child; 4],
        configs : [u8; 4]
    }
    struct Three {
        children : [Child; 3],
        configs : [u8; 3]
    }
    struct Half {
        children : [Child; 2],
        configs : [u8; 2]
    }
    struct Quarter {
        children : [Child; 1],
        configs : [u8; 1]
    }

    //Currently assumes the children are stored in the correct order and not sorted for additional compression and defined by the config.
    enum NodeHandler {
        Leaf(Leaf),
        Full(Full),
        Three(Three),
        Half(Half),
        Quarter(Quarter)
    }

    impl NodeHandler {

        fn has_child(config:u8, child_zorder:usize) -> bool {
            config == (1 << child_zorder) | config
        }

        fn get_child_index(config:u8, child_zorder:usize) -> usize {
            let mut child_mask = 1 << child_zorder;
            if Self::has_child(config, child_zorder) {
                let mut index = 0;
                while child_mask != 1 {
                    child_mask >>= 1;
                    index += 1;
                }
                index
            } else {
                let mut shifts = 0;
                let mut index = 0;
                while child_mask >> shifts != 1 {
                    index += if config >> shifts == 1 { 1 } else { 0 };
                    shifts += 1;
                }
                index
            } 
        }

        //Should always return Ok(), there isn't (*probably) any way for the program to actually return an Err(), but thems the rules.
        pub fn with_set_child(&self, self_config:u8, child_zorder:usize, new_child:Child, child_config:u8) -> Result<Self, ()> {
            let mut new_children;
            let mut new_configs;
            match self {
                Self::Leaf(Leaf{children, configs}) => {
                    new_children = vec![children[0]; 4];
                    new_configs = vec![configs[0]; 4];
                }, 
                Self::Full(Full{children, configs}) => {
                    new_children = Vec::from(children);
                    new_configs = Vec::from(configs);
                },
                Self::Three(Three{children, configs}) => {
                    new_children = Vec::from(children);
                    new_configs = Vec::from(configs);
                },
                Self::Half(Half{children, configs}) => {
                    new_children = Vec::from(children);
                    new_configs = Vec::from(configs);
                },
                Self::Quarter(Quarter{children, configs}) => {
                    new_children = Vec::from(children);
                    new_configs = Vec::from(configs);
                }
            }
            let index = Self::get_child_index(self_config, child_zorder);
            if Self::has_child(self_config, child_zorder) {
                new_children[index] = new_child;
                new_configs[index] = child_config;
            } else {
                new_children.insert(index, new_child);
                new_configs.insert(index, child_config);
            }
            match new_children.len() {
                4 => {
                    let mut is_leaf = true;
                    for i in 0 .. 4 {
                        if new_children[i] != new_child {
                            is_leaf = false; 
                            break
                        }
                    }
                    if is_leaf {
                        Ok ( Self::Leaf(Leaf {
                            children : [new_children[0]],
                            configs : [0b1111]
                        } ) ) 
                    } else {
                        Ok ( Self::Full(Full {
                            children : new_children.try_into().unwrap(),
                            configs : new_configs.try_into().unwrap()
                        } ) )
                    }
                },
                3 => {
                    Ok ( Self::Three(Three {
                        children : new_children.try_into().unwrap(),
                        configs : new_configs.try_into().unwrap()
                    } ) )
                }, 
                2 => {
                    Ok ( Self::Half(Half {
                        children : new_children.try_into().unwrap(),
                        configs : new_configs.try_into().unwrap()
                    } ) )
                }, 
                1 => {
                    Ok ( Self::Quarter(Quarter {
                        children : new_children.try_into().unwrap(),
                        configs : new_configs.try_into().unwrap()
                    } ) )
                },
                _ => {
                    //It should be impossible for this branch to be reached, but Rust demands completeness.
                    Err(())
                }
            }
        }

        //Needs implementation
        //Returns None if the node becomes empty after the operation empty
        pub fn with_removed_child() -> Option<Self> {
            
            None
        }

    }

}

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
// #[derive(PartialEq, Eq, Hash, Clone)]
// pub struct Node {
//     child_indexes: [Index; 4],
// }

// impl Node {

//     fn new(initial_value:Index) -> Self {
//         Self {
//             child_indexes: [initial_value; 4],
//         }
//     }
    
//     //Trusts direction < 4. Will panic if it does
//     fn child(&self, direction:usize) -> Index {
//         self.child_indexes[direction]
//     }

// }

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

    pub fn _raise_root_domain(&mut self, root:Index, path:&Path2D) -> Result<Index, AccessError> {
        let new_root = self.set_node_child(self.empty_root(), path, root)?;
        self.nodes.add_owner(new_root)?;
        self.nodes.remove_owner(root)?;
        Ok(new_root)
    }

    pub fn _lower_root_domain(&mut self, root:Index, path:&Path2D) -> Result<Index, AccessError> {
        let new_root = self.read_destination(root, path)?.index;
        self.nodes.add_owner(new_root)?;
        self.nodes.remove_owner(root)?;
        Ok(new_root)
    }


}

