use std::collections::HashMap;

use macroquad::prelude::scene::Node;
use vec_mem_heap::MemHeap;
pub use vec_mem_heap::{Index, AccessError};


mod node_stuff {
    use super::Index;
    use std::convert::TryInto;
    use std::hash::Hash;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct Child {
        pub index:Index,
        //cycles : u8
    }
    impl Child {
        pub fn new(index:Index, /*pre_cycles:u8*/) -> Self {
            Self {
                index, 
                // pre_cycles
            }
        }
    }

    //We distinguish the structs from the enum vecs because I want data stored contigously (which requires known size arrays instead of Vecs).
    //The structs will be stored in memory (eventually), and the enum below will be used to manipulate them by wrapping the structs so they can be passed around

    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct Leaf {
        children : [Child; 1],
        //Leaves should always have a config (child_mask) of 0b1111 (7)
        configs : [u8; 1]
    }
    impl Leaf {
        pub fn new(child:Child) -> Self {
            Self {
                children : [child],
                configs : [0b1111]
            }
        }
    }
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct Full {
        children : [Child; 4],
        configs : [u8; 4]
    }
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct Three {
        children : [Child; 3],
        configs : [u8; 3]
    }
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct Half {
        children : [Child; 2],
        configs : [u8; 2]
    }
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub struct Quarter {
        children : [Child; 1],
        configs : [u8; 1]
    }

    //Currently assumes the children are stored in the correct order and not sorted for additional compression and defined by the config.
    #[derive(Clone, PartialEq, Eq, Hash)]
    pub enum NodeHandler {
        Leaf(Leaf),
        Full(Full),
        Three(Three),
        Half(Half),
        Quarter(Quarter)
    }

    impl NodeHandler {

        pub fn new_quarter(child:Child, child_config:u8) -> Self {
            Self::Quarter(Quarter {
                children : [child],
                configs : [child_config]
            } )
        }

        pub fn has_child(config:u8, child_zorder:usize) -> bool {
            config == (1 << child_zorder) | config
        }

        pub fn child_index(config:u8, child_zorder:usize) -> usize {
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

        fn to_node(children:Vec<Child>, configs:Vec<u8>) -> Result<Option<Self>, String> {
            match children.len() {
                4 => {
                    let mut is_leaf = true;
                    for i in 0 .. 4 {
                        if children[i] != children[0] {
                            is_leaf = false; 
                            break
                        }
                    }
                    if is_leaf {
                        Ok ( Some ( Self::Leaf(Leaf {
                            children : [children[0]],
                            configs : [0b1111]
                        } ) ) )
                    } else {
                        Ok ( Some ( Self::Full(Full {
                            children : children.try_into().unwrap(),
                            configs : configs.try_into().unwrap()
                        } ) ) )
                    }
                },
                3 => {
                    Ok ( Some ( Self::Three(Three {
                        children : children.try_into().unwrap(),
                        configs : configs.try_into().unwrap()
                    } ) ) )
                }, 
                2 => {
                    Ok ( Some ( Self::Half(Half {
                        children : children.try_into().unwrap(),
                        configs : configs.try_into().unwrap()
                    } ) ) )
                }, 
                1 => {
                    Ok ( Some ( Self::Quarter(Quarter {
                        children : children.try_into().unwrap(),
                        configs : configs.try_into().unwrap()
                    } ) ) )
                },
                0 => Ok(None),
                //This branch should be unreachable
                _ => Err(String::from("Something went very wrong."))
            }
        }

        pub fn read_node(&self) -> (Vec<Child>, Vec<u8>) {
            match self {
                Self::Leaf(Leaf{children, configs}) => {
                    (vec![children[0]; 4], vec![configs[0]; 4])
                }, 
                Self::Full(Full{children, configs}) => {
                    (Vec::from(children), Vec::from(configs))
                },
                Self::Three(Three{children, configs}) => {
                    (Vec::from(children), Vec::from(configs))
                },
                Self::Half(Half{children, configs}) => {
                    (Vec::from(children), Vec::from(configs))
                },
                Self::Quarter(Quarter{children, configs}) => {
                    (Vec::from(children), Vec::from(configs))
                }
            }
        }

        pub fn child(&self, config:u8, child_zorder:usize) -> Option<(Child, u8)> {
            if Self::has_child(config, child_zorder) {
                let index = Self::child_index(config, child_zorder);
                let (children, configs) = self.read_node();
                Some( (children[index], configs[index]) )
            } else {
                None
            }
        }

        //Should always return Ok(Some()), there isn't (*probably) any way for the program to actually return an Err() or an Ok(None), but thems the rules.
        pub fn with_set_child(&self, self_config:u8, child_zorder:usize, new_child:Child, child_config:u8) -> Result<Option<Self>, String> {
            let (mut new_children, mut new_configs) = self.read_node();
            let index = Self::child_index(self_config, child_zorder);
            if Self::has_child(self_config, child_zorder) {
                new_children[index] = new_child;
                new_configs[index] = child_config;
            } else {
                new_children.insert(index, new_child);
                new_configs.insert(index, child_config);
            }
            Self::to_node(new_children, new_configs)
        }

        //Returns None if the node becomes empty after the operation
        pub fn with_removed_child(&self, self_config:u8, child_zorder:usize) -> Result<Option<Self>, String> {
            if !Self::has_child(self_config, child_zorder) {
                return Err(String::from("Can't remove child which doesn't exist"))
            }
            let (mut new_children, mut new_configs) = self.read_node();
            let index = Self::child_index(self_config, child_zorder);
            new_children.remove(index);
            new_configs.remove(index);
            Self::to_node(new_children, new_configs)
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


#[derive(Debug)]
pub struct Location {
    pub index:Index,
    pub config:u8,
    pub depth:usize,
}

use node_stuff::{NodeHandler, Leaf, Child};

//SDAG on steroids
pub struct SparseDirectedGraph {
    nodes : MemHeap<NodeHandler>,
    index_lookup : HashMap<NodeHandler, Index>,
}

impl SparseDirectedGraph {

    pub fn new() -> Self {
                                                //This is such a gross constructor
        let full_node = NodeHandler::Leaf(Leaf::new(Child::new(Index(0))));
        let mut instance = Self {
            nodes : MemHeap::new(),
            index_lookup : HashMap::new()
        };
        instance.add_node(full_node, true);
        instance
    }


    //Private functions used for reading
    fn node(&self, index:Index) -> Result<&NodeHandler, AccessError> {
        self.nodes.data(index)
    }

    fn child(&self, index:Index, zorder:usize, config:u8) -> Result< Option<(Child, u8)>, AccessError> {
        Ok( self.node(index)?.child(config, zorder) )
    }

    //Combine root and initial config
    //Add cyclicity here.
    fn get_trail(&self, root:Index, initial_config:u8, path:&Path2D) -> Result<Vec<(Index, u8)>, AccessError> {
        let mut trail:Vec<(Index, u8)> = vec![(root, initial_config)];
        for step in 0 .. path.directions.len() - 1 {
            let (index, config) = trail[step];
            match self.child(index, path.directions[step], config) {
                Ok( Some ( (child, child_config) ) ) if child.index != index => {
                    trail.push((child.index, child_config));
                },
                Ok(_) => break,
                Err(error) => return Err( error )
            };
        }
        Ok( trail )
    }


    //Private functions used for writing
    fn find_index(&self, node:&NodeHandler) -> Option<Index> {
        self.index_lookup.get(node).copied()
    }

    fn dec_owners(&mut self, index:Index) {
        let mut stack:Vec<Index> = Vec::new();
        stack.push( index );
        while stack.len() != 0 {
            match self.nodes.remove_owner(stack.pop().unwrap()) {
                Ok(Some(node)) => {
                    let (children, _) = node.read_node();
                    for child in children.iter() {
                        stack.push(child.index)
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

    fn add_node(&mut self, node:NodeHandler, protected:bool) -> Index {
        match self.find_index(&node) {
            Some(index) => index,
            None => {
                let node_dup = node.clone();
                let index = self.nodes.push(node, protected);
                self.index_lookup.insert(node_dup, index);
                let (node_kids, _) = self.node(index).unwrap().read_node();
                for child in node_kids {
                    if child.index != index { //Nodes aren't allowed to keep themselves alive.
                        match self.nodes.add_owner(child.index) {
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
    pub fn set_node_child(&mut self, root:Index, initial_config:u8, path:&Path2D, child_index:Index, child_config:u8) -> Result<(Index, u8), AccessError> {
        let trail = self.get_trail(root, initial_config, path)?;
        let mut new_index = child_index;
        let mut new_config = child_config;
        let steps = path.directions.len() - 1;
        for step in 0 ..= steps {
            let (index, config) = if steps - step < trail.len() {
                trail[steps - step]
            } else {
                trail[trail.len() - 1]
            };
            let new_node = match self.node(index) {
                Ok(node) => {
                    match node.with_set_child(
                        config, 
                        path.directions[steps - step], 
                        Child::new(new_index), 
                        new_config
                    )  {
                        Ok ( Some (node) ) => node,
                        //Neither of these two arms should happen.
                        Ok ( None ) => panic!("What?"),
                        Err(error) => panic!("{error}"),
                    }
                },
                Err(AccessError::FreeMemory(_)) => NodeHandler::new_quarter(Child::new(new_index), new_config),
                Err( error ) => return Err( error ),
            };    
            new_index = self.add_node(new_node, false);
            new_config = config;   
        }
        if let Err( error ) = self.nodes.add_owner(new_index) { dbg!(error); }
        self.dec_owners(root);
        Ok( (new_index, new_config) )
    }


    //Public functions used for reading
    pub fn read_destination(&self, root:Index, path:&Path2D, initial_config:u8) -> Result<Location, AccessError> {
        let trail = self.get_trail(root, initial_config, path)?;
        match trail.last() {
            Some((index, config)) => {
                match self.child(*index, path.directions[trail.len() - 1], *config)? {
                    Some( (child, child_config) ) => {
                        Ok( Location {
                            index : child.index,
                            config : child_config, 
                            depth : trail.len() - 1
                        } )
                    }, 
                    None => Err( AccessError::InvalidRequest )
                }
                
            },
            //Can't read from the end of a trail if the trail is empty
            None => Err( AccessError::InvalidRequest )
        }
    }

    pub fn dfs_leaves(&self, root:Index, initial_config:u8) -> Vec<(u32, u32, Index)> {
        let mut leaves = Vec::new();
        let mut stack = Vec::new();
        //         index, configuration, depth, zorder
        stack.push((root, initial_config, 0u32, 0u32));

        while stack.len() != 0 {
            let (cur_index, cur_config, layers_deep, zorder) = stack.pop().unwrap();
            if layers_deep == 10 { //Arbitrary depth catcher to prevent infinite diving
                dbg!(*cur_index);
                continue;
            }
            //Because we're just following pointers this only fails if the structure has failed.
            let (children, child_configs) = self.node(cur_index).unwrap().read_node();
            for child_zorder in 0 .. 4 {
                if !NodeHandler::has_child(cur_config, child_zorder) {
                    continue
                }
                let vec_index = NodeHandler::child_index(cur_config, child_zorder);
                let child_index = children[vec_index].index;
                if child_index == cur_index {
                    leaves.push((zorder, layers_deep, child_index));
                    break
                } else {
                    stack.push((child_index, child_configs[vec_index], layers_deep + 1, (zorder << 2) | child_zorder as u32))
                }
            }
        }
        leaves
    }

    //Public functions used for root manipulation
    pub fn empty_root(&self) -> Index {
        Index(0)
    }

    // pub fn _raise_root_domain(&mut self, root:Index, path:&Path2D) -> Result<Index, AccessError> {
    //     let new_root = self.set_node_child(self.empty_root(), path, root)?;
    //     self.nodes.add_owner(new_root)?;
    //     self.nodes.remove_owner(root)?;
    //     Ok(new_root)
    // }

    // pub fn _lower_root_domain(&mut self, root:Index, path:&Path2D) -> Result<Index, AccessError> {
    //     let new_root = self.read_destination(root, path)?.index;
    //     self.nodes.add_owner(new_root)?;
    //     self.nodes.remove_owner(root)?;
    //     Ok(new_root)
    // }


}

