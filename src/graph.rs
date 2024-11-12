use std::collections::HashMap;
use std::convert::TryInto;
use vec_mem_heap::MemHeap;
pub use vec_mem_heap::{Index, AccessError};


mod node_stuff {
    use super::Index;
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
    pub struct Full {
        pub children : [Child; 4],
        pub configs : [u8; 4]
    }
    impl Full {
        pub fn new_leaf(child:Child) -> Self {
            Self { 
                children : [child; 4],
                configs : [0b0000; 4]
             }
        }
    }
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Three {
        pub children : [Child; 3],
        pub configs : [u8; 3]
    }
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Half {
        pub children : [Child; 2],
        pub configs : [u8; 2]
    }
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Quarter {
        pub children : [Child; 1],
        pub configs : [u8; 1]
    }

    //Currently assumes the children are stored in the correct order and not sorted for additional compression
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum NodeHandler {
        FullNode { 
            lod : Index,
            node : Full
        },
        ThreeNode { 
            lod : Index,
            node : Three
        },
        HalfNode { 
            lod : Index,
            node : Half
        },
        QuarterNode { 
            lod : Index,
            node : Quarter
        },    
    }

    impl NodeHandler {

        //Think of better names for (read and raw) node
        fn read_node(&self, config:u8) -> ([Child; 4], [u8; 4]) {
            let (lod, mut children, mut child_configs) = match self {
                Self::FullNode{lod : _, node} => return (node.children, node.configs),
                Self::ThreeNode{lod, node} => (lod, Vec::from(node.children), Vec::from(node.configs)),
                Self::HalfNode{lod, node} => (lod, Vec::from(node.children), Vec::from(node.configs)),
                Self::QuarterNode{lod, node} => (lod, Vec::from(node.children), Vec::from(node.configs)),
            };
            for i in 0 .. 4 {
                if !Self::has_child(config, i) {
                    children.insert(i, Child::new(*lod));
                    child_configs.insert(i, 0b0000);
                }
            }
            (children.try_into().unwrap(), child_configs.try_into().unwrap())
        }

        pub fn has_child(config:u8, child_zorder:usize) -> bool {
            config & (1 << child_zorder) != 0
        }

        pub fn child(&self, config:u8, child_zorder:usize) -> (Child, u8) {
            let (children, configs) = self.read_node(config);
            (children[child_zorder], configs[child_zorder])
        }

        pub fn raw_node(&self) -> (Index, Vec<Child>, Vec<u8>) {
            match self {
                Self::FullNode{lod, node} => {
                    (*lod, Vec::from(node.children), Vec::from(node.configs))
                },
                Self::ThreeNode{lod, node} => {
                    (*lod, Vec::from(node.children), Vec::from(node.configs))
                },
                Self::HalfNode{lod, node} => {
                    (*lod, Vec::from(node.children), Vec::from(node.configs))
                },
                Self::QuarterNode{lod, node} => {
                    (*lod, Vec::from(node.children), Vec::from(node.configs))
                },
            }
        }

        pub fn lod(&self) -> Index {
            match self {
                Self::FullNode{lod, node : _} => *lod,
                Self::ThreeNode{lod, node : _} => *lod,
                Self::HalfNode{lod, node : _} => *lod,
                Self::QuarterNode{lod, node : _} => *lod,
            }
        }

        pub fn with_different_child(&self, self_config:u8, child_zorder:usize, new_child:Child, child_config:u8) -> ([Child; 4], [u8; 4]) {
            let (mut new_children, mut new_configs) = self.read_node(self_config);
            new_children[child_zorder] = new_child;
            new_configs[child_zorder] = child_config;
            (new_children, new_configs)
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


#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub index:Index,
    pub config:u8,
    pub depth:usize,
}


pub use node_stuff::{NodeHandler, Full, Three, Half, Quarter, Child};
//SDAG on steroids
pub struct SparseDirectedGraph {
    nodes : MemHeap<NodeHandler>,
    index_lookup : HashMap<NodeHandler, Index>,
}

impl SparseDirectedGraph {

    pub fn new() -> Self {
        let air = NodeHandler::FullNode {
            lod : Index(0),
            node : Full::new_leaf(Child::new(Index(0)))
        };
        let solid = NodeHandler::FullNode {
            lod : Index(1),
            node : Full::new_leaf(Child::new(Index(1)))
        };        
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

    //Add cyclicity here
    fn get_trail(&self, root:Location, path:&Path2D) -> Result< Vec<(Index, u8)> , AccessError> {
        let mut trail:Vec<(Index, u8)> = vec![(root.index, root.config)];
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

    //Fix the lod_valing here
    fn compact_node_parts(&self, children:[Child; 4], configs:[u8; 4]) -> Result<(NodeHandler, u8), AccessError> {
        let lod_potentials = [
            self.node(children[0].index)?.lod(),
            self.node(children[1].index)?.lod(),
            self.node(children[2].index)?.lod(),
            self.node(children[3].index)?.lod()
            ];
            //Once we're sorting the children just loop through list, counting then switching when it changes
        let lod_val = if lod_potentials[0] == lod_potentials[1] || lod_potentials[0] == lod_potentials[2] || lod_potentials[0] == lod_potentials[3]  {
            lod_potentials[0]
        } else if lod_potentials[1] == lod_potentials[2] || lod_potentials[1] == lod_potentials[3] {
            lod_potentials[1]
        } else {
            lod_potentials[3]
        };
        let mut culled_children = Vec::new();
        let mut culled_configs = Vec::new();
        let mut node_config = 0;
        for index in 0 .. 4 {
            if children[index].index != lod_val {
                culled_children.push(children[index]);
                culled_configs.push(configs[index]);
                node_config |= 1 << index;
            } 
        }
        let node = match culled_children.len() {
            4 => Ok ( NodeHandler::FullNode {
                    lod : lod_val,
                    node : Full {
                        children : culled_children.try_into().unwrap(),
                        configs : culled_configs.try_into().unwrap()
                    }
                } ),
            3 => Ok ( NodeHandler::ThreeNode {
                    lod : lod_val,
                    node : Three {
                        children : culled_children.try_into().unwrap(),
                        configs : culled_configs.try_into().unwrap()
                    }
                } ), 
            2 => Ok ( NodeHandler::HalfNode {
                lod : lod_val,
                node : Half {
                    children : culled_children.try_into().unwrap(),
                    configs : culled_configs.try_into().unwrap()
                }
            } ), 
            1 => Ok ( NodeHandler::QuarterNode {
                lod : lod_val,
                node : Quarter {
                    children : culled_children.try_into().unwrap(),
                    configs : culled_configs.try_into().unwrap()
                }
            } ), 
            0 => Ok ( NodeHandler::FullNode {
                lod : lod_val,
                node : Full {
                    children : [Child::new(lod_val); 4],
                    configs : [0; 4],
                } 
            } ),
            _ => Err ( AccessError::InvalidRequest )
        };
        Ok((node?, node_config))
    }

    //Public functions used for writing
    //Add cyclicity here.
    //The conversion from full to leaf is breaking everything
    pub fn set_node_child(&mut self, root:Location, path:&Path2D, index:Index, config:u8) -> Result<Location, AccessError> {
        let trail = self.get_trail(root, path)?;
        let mut new_index = index;
        let mut new_config = config;
        let steps = path.directions.len() - 1;
        for step in 0 ..= steps {
            let (child_index, child_config) = if steps - step < trail.len() {
                trail[steps - step]
            } else {
                trail[trail.len() - 1]
            };
            if new_index == child_index {
                return if step == steps {
                    Ok ( Location {
                        index : new_index,
                        config : new_config,
                        depth : 0
                    } )
                } else {
                    Ok( root )
                }
            }
            let (new_node, next_config) =  {
                let (children, configs) = self.node(child_index)?
                .with_different_child(
                    child_config, 
                    path.directions[steps - step], 
                    Child::new(new_index), 
                    new_config
                );
                self.compact_node_parts(children, configs)?
            };
            new_index = match self.nodes.status(child_index)? {
                vec_mem_heap::Ownership::Fine(count) if count == 1 => {
                   self.nodes.replace(child_index, new_node)?;
                   child_index
                }
                _ => self.add_node(new_node, false),
            };
            new_config = next_config;
        }
        if let Err( error ) = self.nodes.add_owner(new_index) { dbg!(error); }
        self.dec_owners(root.index);
        Ok ( Location {
            index : new_index,
            config : new_config,
            depth : 0
        } )
    }


    //Public functions used for reading
    pub fn read_destination(&self, root:Location, path:&Path2D) -> Result<Location, AccessError> {
        let trail = self.get_trail(root, path)?;
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

    //Add cyclicity here.
    pub fn dfs_leaves(&self, root:Index, initial_config:u8) -> Vec<(u32, u32, Index)> {
        //Arbitrary limit
        let maximum_render_depth = 10;
        let mut leaves = Vec::new();
        let mut stack = Vec::new();
        //         index, configuration, depth, zorder
        stack.push((root, initial_config, 0u32, 0u32));

        while stack.len() != 0 {
            let (cur_index, cur_config, layers_deep, zorder) = stack.pop().unwrap();
            let (lod, children, child_configs) = self.node(cur_index).unwrap().raw_node();
            let mut ignored = 0;
            for i in 0 .. 4 {
                //If we're in a leaf
                if cur_config == 0 {
                    leaves.push((zorder, layers_deep, lod));
                    break
                //If we're in a cell culled from lod compression
                } else if !NodeHandler::has_child(cur_config, i) && layers_deep + 1 <= maximum_render_depth {
                    leaves.push(((zorder << 2) | i as u32, layers_deep + 1, lod));
                    ignored += 1;
                    continue
                //Keep diving
                } else {
                    if layers_deep + 1 > maximum_render_depth { 
                        println!("Graph exceeds depth limit at index {}, rendering at layer {maximum_render_depth}", *cur_index);
                        leaves.push((zorder, layers_deep, lod));
                    } else {
                        stack.push((children[i - ignored].index, child_configs[i - ignored], layers_deep + 1, (zorder << 2) | i as u32))
                    }
                }
            }
        }
        leaves
    }

    
    //Public functions used for root manipulation
    pub fn empty_root(&self) -> Location {
        Location {
            index : Index(0),
            config : 0b0000,
            depth : 0,
        }
    }

    //Figure these two out with the new system
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
