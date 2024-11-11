use std::collections::HashMap;

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

    //The structs will be stored in memory (eventually) and the enum will be used to manipulate them

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Leaf {
        pub child : Child
    }
    impl Leaf {
        pub fn new(child:Child) -> Self {
            Self { child }
        }
    }
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Three {
        pub lod : Child,
        children : [Child; 3],
        configs : [u8; 3]
    }
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Half {
        pub lod : Child,
        children : [Child; 2],
        configs : [u8; 2]
    }
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Quarter {
        pub lod : Child,
        children : [Child; 1],
        configs : [u8; 1]
    }

    //Currently assumes the children are stored in the correct order and not sorted for additional compression
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum NodeHandler {
        Leaf(Leaf),
        Three(Three),
        Half(Half),
        Quarter(Quarter)
    }

    impl NodeHandler {
/*
        fn has_child(config:u8, child_zorder:usize) -> bool {
            config == (1 << child_zorder) | config
        }

        fn child_index(config:u8, child_zorder:usize) -> usize {
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
*/
        //Think of better names for (read and raw) node
        fn read_node(&self, config:u8) -> ([Child; 4], [u8; 4]) {
            let (lod, mut children, mut child_configs) = match self {
                Self::Leaf(Leaf{child}) => {
                    return ([*child; 4], [0b0000; 4])
                }, 
                Self::Three(Three{lod, children, configs}) => {
                    (lod, Vec::from(children), Vec::from(configs))
                },
                Self::Half(Half{lod, children, configs}) => {
                    (lod, Vec::from(children), Vec::from(configs))
                },
                Self::Quarter(Quarter{lod, children, configs}) => {
                    (lod, Vec::from(children), Vec::from(configs))
                },
            };
            for i in 0 .. 4 {
                if config & (1 << i) == 0 {
                    children.insert(i, *lod);
                    child_configs.insert(i, 0b0000);
                }
            }
            (children.try_into().unwrap(), child_configs.try_into().unwrap())
        }

        fn to_node(children:[Child; 4], configs:[u8; 4]) -> Result<(Self, u8), String> {
            //Once we're sorting the children just loop through list, counting then switching when it changes
            let lod_val = if children[0] == children[1] || children[0] == children[2] || children[0] == children[3]  {
                children[0]
            } else if children[1] == children[2] || children[1] == children[3] {
                children[1]
            } else {
                children[3]
            };
            let mut culled_children = Vec::new();
            let mut culled_configs = Vec::new();
            let mut node_config = 0;
            for index in 0 .. 4 {
                if children[index] != lod_val {
                    culled_children.push(children[index]);
                    culled_configs.push(configs[index]);
                    node_config |= 1 << index;
                } 
            }
            let node = match culled_children.len() {
                3 => {
                    Ok ( Self::Three(Three {
                        lod : lod_val,
                        children : culled_children.try_into().unwrap(),
                        configs : culled_configs.try_into().unwrap()
                    } ) )
                }, 
                2 => {
                    Ok ( Self::Half(Half {
                        lod : lod_val,
                        children : culled_children.try_into().unwrap(),
                        configs : culled_configs.try_into().unwrap()
                    } ) )
                }, 
                1 => {
                    Ok ( Self::Quarter(Quarter {
                        lod : lod_val,
                        children : culled_children.try_into().unwrap(),
                        configs : culled_configs.try_into().unwrap()
                    } ) )
                },
                0 => Ok ( Self::Leaf( Leaf::new(lod_val) ) ),
                //This branch should be unreachable
                _ => Err ( String::from("Something went very wrong.") )
            };
            Ok((node?, node_config))
        }

        pub fn child(&self, config:u8, child_zorder:usize) -> (Child, u8) {
            let (children, configs) = self.read_node(config);
            (children[child_zorder], configs[child_zorder])
        }

        pub fn raw_node(&self) -> (Child, Vec<Child>, Vec<u8>) {
            match self {
                Self::Leaf(Leaf{child}) => {
                    (*child, Vec::from([*child]), vec![0b0000])
                }, 
                Self::Three(Three{lod, children, configs}) => {
                    (*lod, Vec::from(children), Vec::from(configs))
                },
                Self::Half(Half{lod, children, configs}) => {
                    (*lod, Vec::from(children), Vec::from(configs))
                },
                Self::Quarter(Quarter{lod, children, configs}) => {
                    (*lod, Vec::from(children), Vec::from(configs))
                },
            }
        }

        //Safe to unwrap, should always return Ok(_)
        pub fn with_different_child(&self, self_config:u8, child_zorder:usize, new_child:Child, child_config:u8) -> Result<(Self, u8), String> {
            let (mut new_children, mut new_configs) = self.read_node(self_config);
            new_children[child_zorder] = new_child;
            new_configs[child_zorder] = child_config;
            Self::to_node(new_children, new_configs)
        }

        pub fn lod(&self) -> Child   {
            match self {
                NodeHandler::Three(node) => node.lod,
                NodeHandler::Half(node) => node.lod,
                NodeHandler::Quarter(node) => node.lod,
                NodeHandler::Leaf(node) => node.child,
            }
        }

        pub fn assign_lod(&mut self, lod:Child) {
            match self {
                NodeHandler::Three(node) => node.lod = lod,
                NodeHandler::Half(node) => node.lod = lod,
                NodeHandler::Quarter(node) => node.lod = lod,
                NodeHandler::Leaf(node) => node.child = lod,
            }
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
        let air = NodeHandler::Leaf(Leaf::new(Child::new(Index(0))));
        let solid = NodeHandler::Leaf(Leaf::new(Child::new(Index(1))));
        let mut instance = Self {
            nodes : MemHeap::new(),
            index_lookup : HashMap::new()
        };
        instance.add_node(air, true);
        instance.add_node(solid, true);
        instance
    }


    //Private functions used for reading
    fn node(&self, index:Index) -> Result<&NodeHandler, AccessError> {
        self.nodes.data(index)
    }

    fn child(&self, index:Index, zorder:usize, config:u8) -> Result<(Child, u8), AccessError> {
        Ok( self.node(index)?.child(config, zorder) )
    }

    //Add cyclicity here.
    fn get_trail(&self, root:Index, initial_config:u8, path:&Path2D) -> Result< Vec<(Index, u8)> , AccessError> {
        let mut trail:Vec<(Index, u8)> = vec![(root, initial_config)];
        for step in 0 .. path.directions.len() - 1 {
            let (index, config) = trail[step];
            match self.child(index, path.directions[step], config) {
                Ok( (child, child_config) ) if child.index != index => {
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
        let mut stack = Vec::new();
        stack.push( index );
        while stack.len() != 0 {
            match self.nodes.remove_owner(stack.pop().unwrap()) {
                Ok(Some(node)) => {
                    let (_, children, _) = node.raw_node();
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
                let (_, node_kids, _) = self.node(index).unwrap().raw_node();
                for child in node_kids {
                    //Nodes aren't allowed to keep themselves alive.
                    if child.index != index { 
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
    pub fn set_node_child(&mut self, root:Index, initial_config:u8, path:&Path2D, index:Index, config:u8) -> Result<(Index, u8), AccessError> {
        let trail = self.get_trail(root, initial_config, path)?;
        let mut new_index = index;
        let mut new_config = config;
        let steps = path.directions.len() - 1;
        for step in 0 ..= steps {
            let (child_index, child_config) = if steps - step < trail.len() {
                trail[steps - step]
            } else {
                trail[trail.len() - 1]
            };
            let (mut new_node, next_config) = match self.node(child_index) {
                Ok(node) => { 
                    match node.with_different_child(
                        child_config, 
                        path.directions[steps - step], 
                        Child::new(new_index), 
                        new_config
                    )  {
                        Ok ( node) => node,
                        //Should never happen
                        Err(error) => panic!("{error}"),
                    } 
                },
                Err( error ) => return Err( error ),
            };
            //Because of limited data, lod can only be assigned to a child of node.
            //But we promise the child of node will have a valid lod, so we carry that one up.
            let new_lod = self.node(new_node.lod().index)?.lod();
            new_node.assign_lod(new_lod);
            new_index = self.add_node(new_node, false);
            new_config = next_config;   
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
                let (child, child_config) = self.child(*index, path.directions[trail.len() - 1], *config)?;
                Ok( Location {
                    index : child.index,
                    config : child_config, 
                    depth : trail.len() - 1
                } )
            },  
            //Can't read from the end of a trail if the trail is empty
            None => Err( AccessError::InvalidRequest )
        }
    }

    pub fn dfs_leaves(&self, root:Index, initial_config:u8) -> Vec<(u32, u32, Index)> {
        let maximum_render_depth = 10;
        let mut leaves = Vec::new();
        let mut stack = Vec::new();
        //         index, configuration, depth, zorder
        stack.push((root, initial_config, 0u32, 0u32));

        while stack.len() != 0 {
            let (cur_index, cur_config, layers_deep, zorder) = stack.pop().unwrap();
            let (lod, children, child_configs) = self.node(cur_index).unwrap().raw_node();
            //Arbitrary depth catcher to prevent infinite diving
            let mut ignored = 0;
            for i in 0 .. 4 {
                //If we're in a cell which was culled from LOD compression
                if cur_config == 0 {
                    leaves.push((zorder, layers_deep, lod.index));
                    break
                } else if cur_config & (1 << i) == 0 && layers_deep + 1 <= maximum_render_depth {
                    leaves.push(((zorder << 2) | i as u32, layers_deep + 1, lod.index));
                    ignored += 1;
                    continue
                } else {
                    if layers_deep + 1 > maximum_render_depth { 
                        println!("Graph exceeds depth limit at index {}, rendering at layer {maximum_render_depth}", *cur_index);
                        leaves.push((zorder, layers_deep, lod.index));
                    } else {
                        stack.push((children[i - ignored].index, child_configs[i - ignored], layers_deep + 1, (zorder << 2) | i as u32))
                    }
        
                }
            }
        }
        leaves
    }

    //Public functions used for root manipulation
    pub fn empty_root(&self) -> (Index, u8) {
        (Index(0), 0b0000)
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
