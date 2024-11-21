use std::collections::HashMap;
use vec_mem_heap::{MemHeap, Ownership};
pub use vec_mem_heap::{Index, AccessError};


mod node_stuff {
    use super::Index;
    use std::hash::Hash;

    #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
    pub struct NodePointer { 
        pub index : Index,
        pub mask : u8
    }
    impl NodePointer {
        pub fn new(index:Index, mask:u8) -> Self {
            Self {
                index, 
                mask
            }
        }
    }

    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Leaf {
        pub most_common_child : NodePointer,
    }
    impl Leaf {
        pub fn new(index:Index) -> Self {
            Self { 
                most_common_child : NodePointer::new(index, 0b0000) 
            }
        }
    }
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Three {
        pub most_common_child : NodePointer,
        pub other_children : [NodePointer; 3],
    }
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Half {
        pub most_common_child : NodePointer,
        pub other_children : [NodePointer; 2],
    }
    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub struct Quarter {
        pub most_common_child : NodePointer,
        pub other_children : NodePointer,
    }


    #[derive(Clone, PartialEq, Eq, Hash, Debug)]
    pub enum NodeHandler {
        LeafNode(Leaf),
        ThreeNode(Three),
        HalfNode(Half),
        QuarterNode(Quarter),  
    }

    impl NodeHandler {

        fn read_node(&self, mask:u8) -> [NodePointer; 4] {
            let (lod, mut children) = self.raw_node();
            for i in 0 .. 4 {
                if !Self::has_child(mask, i) {
                    children.insert(i, lod);
                }
            }
            children.try_into().unwrap()
        }

        pub fn has_child(mask:u8, child_zorder:usize) -> bool {
            (mask >> child_zorder) & 1 == 1
        }

        pub fn child(&self, mask:u8, child_zorder:usize) -> NodePointer {
            self.read_node(mask)[child_zorder]
        }

        pub fn raw_node(&self) -> (NodePointer, Vec<NodePointer>) {
            match self {
                Self::LeafNode(node) => (node.most_common_child, Vec::new()),
                Self::ThreeNode(node) => (node.most_common_child, Vec::from(node.other_children)),
                Self::HalfNode(node) => (node.most_common_child, Vec::from(node.other_children)),
                Self::QuarterNode(node) => (node.most_common_child, Vec::from([node.other_children])),
            }
        }

        pub fn with_different_child(&self, mask:u8, child_zorder:usize, new_child:NodePointer) -> (Self, u8) {
            let mut new_children = self.read_node(mask);
            new_children[child_zorder] = new_child;
            NodeHandler::compact_node_parts(new_children)
        }

        fn compact_node_parts( children:[NodePointer; 4]) -> (Self, u8) {
            let lod_zorder = if children[0] == children[1] || children[0] == children[2] || children[0] == children[3]  {
                0
            } else if children[1] == children[2] || children[1] == children[3] {
                1
            } else if children[2] == children[3] {
                2
            } else {
                3
            };
            let mut culled_children = Vec::new();
            let mut mask = 0;
            for zorder in 0 .. 4 {
                if children[zorder] != children[lod_zorder] {
                    culled_children.push(children[zorder]);
                    mask |= 1 << zorder;
                } 
            }
            ( match culled_children.len() {
                3 => Self::ThreeNode(Three{
                    most_common_child : children[lod_zorder],
                    other_children : [culled_children[0], culled_children[1], culled_children[2]]
                } ), 
                2 => Self::HalfNode(Half{
                    most_common_child : children[lod_zorder],
                    other_children : [culled_children[0], culled_children[1]]
                } ), 
                1 => Self::QuarterNode(Quarter{
                    most_common_child : children[lod_zorder],
                    other_children : culled_children[0]
                } ), 
                0 => Self::LeafNode(Leaf{
                    most_common_child : children[0]
                } ),
                _ => panic!("What?")
            }, 
            mask)
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



pub use node_stuff::{NodeHandler, Leaf, NodePointer};
pub struct SparseDirectedGraph {
    nodes : MemHeap<NodeHandler>,
    index_lookup : HashMap<NodeHandler, Index>,
}

impl SparseDirectedGraph {

    pub fn new() -> Self {
        let empty = NodeHandler::LeafNode(Leaf::new(Index(0)));
        let full = NodeHandler::LeafNode(Leaf::new(Index(1)));
        let mut instance = Self {
            nodes : MemHeap::new(),
            index_lookup : HashMap::new(),
        };
        instance.add_node(empty, true);
        instance.add_node(full, true);
        instance
    }


    //Private functions used for reading
    fn node(&self, index:Index) -> Result<&NodeHandler, AccessError> {
        self.nodes.data(index)
    }

    fn child(&self, node:NodePointer, zorder:usize) -> Result<NodePointer, AccessError> {
        Ok( self.node(node.index)?.child(node.mask, zorder) )
    }

    //Add cyclicity here
    fn get_trail(&self, root:NodePointer, path:&Path2D) -> Result< Vec<NodePointer> , AccessError> {
        let mut trail = vec![root];
        for step in 0 .. path.directions.len() - 1 {
            let parent = trail[step];
            match self.child(parent, path.directions[step]) {
                Ok( child ) if child != parent => trail.push(child),
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
                    let (lod, children) = node.raw_node();
                    stack.push(lod.index);
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
                let (lod, node_kids) = self.node(index).unwrap().raw_node();
                match self.nodes.add_owner(lod.index) {
                    Ok(_) | Err( AccessError::ProtectedMemory(_) ) => (),
                    Err( error ) => { dbg!(error); () }
                }
                for child in node_kids {
                    match self.nodes.add_owner(child.index) {
                        Ok(_) | Err( AccessError::ProtectedMemory(_) ) => (),
                        Err( error ) => { dbg!(error); () }
                    }
                }
                index
            }
        }
    }

    //Public functions used for writing
    //Add cyclicity here.
    pub fn set_node_child(&mut self, root:NodePointer, path:&Path2D, new_child:NodePointer) -> Result<NodePointer, AccessError> {
        let trail = self.get_trail(root, path)?;
        let mut cur_child = new_child;
        let steps = path.directions.len() - 1;
        for step in 0 ..= steps {
            let parent = if steps - step < trail.len() {
                trail[steps - step]
            } else {
                trail[trail.len() - 1]
            };
            let (new_node, new_mask) =  {
                self.node(parent.index)?.with_different_child(
                    parent.mask, 
                    path.directions[steps - step], 
                    cur_child
                )
            };
            dbg!(&new_node);
            cur_child.index = self.add_node(new_node, false);
            cur_child.mask = new_mask;
        }
        if let Err( error ) = self.nodes.add_owner(cur_child.index) { dbg!(error); }
        self.dec_owners(root.index);
        Ok ( cur_child )
    }


    //Public functions used for reading
    pub fn read_destination(&self, root:NodePointer, path:&Path2D) -> Result<(NodePointer, u32), AccessError> {
        let trail = self.get_trail(root, path)?;
        match trail.last() {
            Some(node_pointer) => {
                let child = self.child(*node_pointer, path.directions[trail.len() - 1])?;
                Ok( (child, trail.len() as u32) )
            },  
            //Can't read from the end of a trail if the trail is empty
            None => Err( AccessError::InvalidRequest )
        }
    }

    //Add cyclicity here.
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
            let (most_common_child, children) = self.node(node.index).unwrap().raw_node();
            let mut ignored = 0;
            //If we're cycling
            if node.mask == 0 && (most_common_child.index == node.index ) {
                leaves.push((zorder, layers_deep, most_common_child.index));
                continue
            }
            for i in 0 .. 4 {
                if layers_deep + 1 > maximum_render_depth { 
                    println!("Graph exceeds depth limit at index {}, rendering at layer {maximum_render_depth}", *node.index);
                } else if !NodeHandler::has_child(node.mask, i) {
                    stack.push((most_common_child, layers_deep + 1, (zorder << 2) | i as u32));
                    ignored += 1;
                } else {
                    stack.push((children[i - ignored], layers_deep + 1, (zorder << 2) | i as u32));
                }
            }
        }
        leaves
    }

    //Public functions used for root manipulation
    pub fn get_root(&self, leaf:usize) -> NodePointer {
        NodePointer {
            index : Index(leaf),
            mask : 0b0000,
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


    pub fn profile(&self) {
        println!("---------- STATUS ----------");
        let mut free_memory = 0;
        let mut reserved_memory = 0;
        let mut dangling = 0;
        let mut types = [0; 4];
        for index in 0 .. self.nodes.length() {
            let cur_index = Index(index);
            if let Ok(status) = self.nodes.status(cur_index) {
                if let Ownership::Fine(_) = status {
                    reserved_memory += 1;
                    if let Ok(node) = self.node(cur_index) {
                        let (_, children) = node.raw_node();
                        types[children.len()] += 1;
                    }
                } else { dangling += 1 }
            } else { free_memory += 1 }
        }

        println!("There are {} dynamic nodes within the tree, consisting of:", reserved_memory);
        println!("{} leaves, {} quarters, {} halves, and {} threes", types[0], types[1], types[2], types[3]);
        println!("{} dangling nodes and {} free slots", dangling, free_memory);

    }

}
