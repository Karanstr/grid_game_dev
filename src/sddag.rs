use macroquad::math::*;


//Make sure we modify this when changing dimension
#[derive(Debug)]
pub struct Node {
    pub child_indexes:[usize; 4],
    pub ref_count:u8,
}

impl Node {
    fn new_empty() -> Self {
        Self {
            child_indexes: [0, 0, 0, 0],
            ref_count: 0
        }
    }

    fn same_children(node1:&Node, node2:&Node) -> bool {
        node1.child_indexes == node2.child_indexes
    }

    fn count_kids_and_get_last(&self) -> (u32, usize) {
        let mut count:u32 = 0;
        let mut last_set_kid = 0;
        for child in 0 .. self.child_indexes.len() {
            if self.child_indexes[child] != 0 {
                last_set_kid = child;
                count += 1;
            }
        }
        (count, last_set_kid)
    }

    fn clone_without_ref(&self) -> Self {
        Self {
            child_indexes: self.child_indexes,
            ref_count: 0
        }
    }
    
}


#[derive(Clone, Debug)]
pub struct NodeAddress {
    pub layer:usize,
    pub index:usize
}

impl NodeAddress {
    
    pub fn new(layer:usize, index:usize) -> Self {
        Self {
            layer,
            index
        }
    }

}

//Stores dimension for debugging
#[derive(Debug)]
pub struct Path {
    _dimension : u32,
    directions : Vec<u8>
} 

impl Path {
    pub fn from(bit_path:u32, steps:usize, dimension:u32) -> Self {
        let mut directions:Vec<u8> = Vec::with_capacity(steps);
        let mut mask:u32 = 0;
        for _ in 0 .. dimension { mask = (mask << 1) | 1 }
        for step in 0 .. steps {
            let cur_direction = bit_path >> dimension*(steps - step - 1) as u32 & mask;
            directions.push(cur_direction as u8)
        }
        Self {
            _dimension : dimension,
            directions
        }
    }
}


pub struct SparseDimensionlessDAG {
    pub node_pot:Vec<Vec<Node>>
}

impl SparseDimensionlessDAG {

    pub fn new(max_layer:usize) -> Self {
        let mut new_self = Self {
            node_pot : Vec::new()
        };
        new_self.ensure_depth(max_layer);
        new_self
    }

    
    // //These don't belong here? they should be under the NodeAddress class or smthing?
    // //Public methods used to modify root nodes
    // pub fn shrink_root_to_fit(&mut self, root:&mut NodeAddress) -> IVec2 {
    //     //Descend to single filled child
    //     let mut blocks_shifted = IVec2::new(0, 0);
    //     loop {
    //         if root.layer == 0 { return blocks_shifted }
    //         let (kid_count, last_index) = self.get_node(&root).count_kids_and_get_last();
    //         if kid_count != 1 { break }
    //         blocks_shifted += self.lower_root_by_one(root, last_index);
    //     }
    //     // loop {
    //     //     if root.layer == 0 { return blocks_shifted }
    //     //     if self.compact_root_children(root) == false {
    //     //         break
    //     //     }
    //     // }
    //     blocks_shifted
    // }
    
    // pub fn raise_root_by_one(&mut self, root:&mut NodeAddress, from_quadrant:usize) -> IVec2 {
    //     //Limit to prevent binarization from overflowing
    //     if root.layer == MAXLAYER2D { return IVec2::ZERO }
    //     let shift_directions = [
    //         IVec2::new(1, 1),  //Expanding out of (-1, -1)
    //         IVec2::new(-1, 1), //Expanding out of (1, -1)
    //         IVec2::new(1, -1),  //Expanding out of (-1, 1)
    //         IVec2::new(-1, -1), //Expanding out of (1, 1)
    //     ];
    //     self.ensure_depth(root.layer + 1);
    //     let mut new_root = NodeAddress::new(root.layer + 1, 0);
    //     self.set_node_child(&mut new_root, root.layer + 1, from_quadrant, root.index);
    //     root.layer = new_root.layer;
    //     root.index = new_root.index;
    //     let blocks_per_dimension = 2i32.pow(root.layer as u32 - 1);
    //     IVec2::splat(blocks_per_dimension) * shift_directions[from_quadrant]
    // }

    // pub fn lower_root_by_one(&mut self, root:&mut NodeAddress, preserve_quadrant:usize) -> IVec2 {
    //     if root.layer == 0 { return IVec2::ZERO }
    //     let shift_directions = [
    //         IVec2::new(-1, -1),  //Preserves (-1, -1)
    //         IVec2::new(1, -1), //Preserves (1, -1)
    //         IVec2::new(-1, 1),  //Preserves (-1, 1)
    //         IVec2::new(1, 1), //Preserves (1, 1)
    //     ];
    //     let child_index = self.get_node_child_index(&root, preserve_quadrant);
    //     self.transfer_reference(&root, &NodeAddress::new(root.layer - 1, child_index));
    //     root.layer = root.layer - 1;
    //     root.index = child_index;
    //     let blocks_per_dimension = 2i32.pow(root.layer as u32);
    //     IVec2::splat(blocks_per_dimension) * shift_directions[preserve_quadrant]
    // }

    // //We're doing this one last, not 2d yet
    // fn _compact_root_children(&mut self, root:&mut NodeAddress) -> bool {
    //     let child_directions = [1, 0];
    //     let mut new_root_node = Node::new_empty();
    //     let children = self.get_node(&root).child_indexes;
    //     for index in 0 .. child_directions.len() {
    //         let address = NodeAddress::new(root.layer - 1, children[index]);
    //         let node = self.get_node(&address);
    //         let (child_count, last_index) = node.count_kids_and_get_last();
    //         if child_count > 1 || last_index != child_directions[index] {
    //             return false //Cannot compact root
    //         }
    //         new_root_node.child_indexes[index] = node.child_indexes[child_directions[index]];
    //     } //If we don't terminate we are safe to lower the root
    //     let new_root_index = self.add_node(root.layer - 1, new_root_node);
    //     self.transfer_reference(&root, &NodeAddress::new(root.layer - 1, new_root_index));
    //     root.layer -= 1;
    //     root.index = new_root_index;
    //     true //Successfully compacted root
    // }


    //Private methods used to.. 
    fn ensure_depth(&mut self, layer:usize) {
        for _i in self.node_pot.len()..=layer {
            self.node_pot.push(vec![Node::new_empty()]);
        }
    }


    //Private methods used to read from the dag
    fn get_mut_node(&mut self, address:&NodeAddress) -> &mut Node {
        &mut self.node_pot[address.layer][address.index]
    }

    fn get_node_child_index(&self, address:&NodeAddress, child_direction:usize) -> usize {
        self.node_pot[address.layer][address.index].child_indexes[child_direction]
    }

    fn find_node_on_layer(&self, node:&Node, layer:usize) -> usize {
        let mut cur_index = self.node_pot[layer].len() - 1;
        //We loop backwards so when searching for an empty node we hit the empty index (0) last
        for node_from_pot in self.node_pot[layer].iter().rev() {
            if Node::same_children(node, node_from_pot) {
                break
            }
            if cur_index != 0 {
                cur_index -= 1
            }
        }
        cur_index
    }

    fn get_or_make_empty_index(&mut self, layer:usize) -> usize {
        let empty_node = Node::new_empty();
        let mut first_avaliable_index = self.find_node_on_layer(&empty_node, layer);
        if first_avaliable_index == 0 {
            first_avaliable_index = self.node_pot[layer].len();
            self.node_pot[layer].push(empty_node);
        }
        first_avaliable_index
    }

    fn get_trail(&self, root:&NodeAddress, path:&Path) -> Vec<usize> {
        let mut trail:Vec<usize> = Vec::with_capacity(path.directions.len() + 1);
        trail.push(root.index); //We start our journey at the root
        for step in 0 .. path.directions.len() - 1 {
            let cur_address= NodeAddress::new(root.layer - step, trail[step]);
            let child_direction = path.directions[step] as usize;
            trail.push(self.get_node_child_index(&cur_address, child_direction));
        }
        trail
    }


    //Public methods used to read from the dag
    pub fn get_node(&self, address:&NodeAddress) -> &Node {
        &self.node_pot[address.layer][address.index]
    }

    pub fn read_node_child(&self, root:&NodeAddress, node_layer:usize, path:&Path) -> usize {
        let trail = self.get_trail(&root, path);
        let address = NodeAddress::new(node_layer, *trail.last().unwrap());
        self.get_node_child_index(&address, *path.directions.last().unwrap() as usize)
    }

    //Works with cell_counts of up to 64. More than that and the u64 overflows.
    pub fn df_to_binary(&self, root:&NodeAddress, dimensions:u32) -> u64 {
        let mut resulting_binary:u64 = 0;
        let mut queue: Vec<(NodeAddress, u32)> = Vec::new();
        queue.push((root.clone(), 0));

        while queue.len() != 0 {
            let (cur_address, cur_bin_path) = queue.pop().unwrap();
            let cur_node = self.get_node(&cur_address);

            for child in 0..cur_node.child_indexes.len() {
                let child_index = cur_node.child_indexes[child];
                let child_path = (cur_bin_path << dimensions) | child as u32;
                if child_index == 0 { continue }
                else if cur_address.layer != 0 { 
                    queue.push( (NodeAddress::new(cur_address.layer - 1, child_index), child_path) )
                } else {
                    resulting_binary |= 1 << child_path;
                }
                
            }
        }
        resulting_binary
    }


    //Private methods used to modify dag data
    fn dec_ref_count(&mut self, address:&NodeAddress) {
        let mut queue:Vec<NodeAddress> = Vec::new();
        if address.index != 0 { queue.push(address.clone()) }
        while queue.len() != 0 {
            let cur_address = queue.pop().unwrap();
            let cur_node = self.get_mut_node(&cur_address);
            cur_node.ref_count -= 1;
            //If node needs to be deleted
            if cur_node.ref_count == 0 {
                //If has children nodes
                if cur_address.layer != 0 {
                    for index in cur_node.child_indexes.iter() {
                        //If child isn't a null node
                        if *index != 0 {
                            queue.push(NodeAddress::new(cur_address.layer - 1, *index));
                        }
                    }
                } //Free cur_node
                self.node_pot[cur_address.layer][cur_address.index] = Node::new_empty();
            }
        }
    }

    fn inc_ref_count(&mut self, address:&NodeAddress) {
        if address.index != 0 {
            self.get_mut_node(&address).ref_count += 1;
        }
    }

    fn add_node(&mut self, layer:usize, node:Node) -> usize {
        if Node::same_children(&node, &Node::new_empty()) {
            return 0
        }
        let mut index = self.find_node_on_layer(&node, layer);
        if index == 0 { //Node is unique
            index = self.get_or_make_empty_index(layer);
            if layer != 0 {
                for child in node.child_indexes.iter() {
                    if *child != 0 {
                        self.node_pot[layer - 1][*child].ref_count += 1;
                    }
                }
            }
            self.node_pot[layer][index] = node;
        }
        index
    }

    fn add_modified_node(&mut self, address:&NodeAddress, child_direction:usize, new_index:usize) -> usize {
        let mut mod_node = self.get_node(&address).clone_without_ref();
        mod_node.child_indexes[child_direction] = new_index;
        self.add_node(address.layer, mod_node)
    }

    fn transfer_reference(&mut self, giver:&NodeAddress, reciever:&NodeAddress) {
        self.inc_ref_count(reciever);
        self.dec_ref_count(giver);
    }


    //Public methods used to modify dag data
    pub fn set_node_child(&mut self, root:&mut NodeAddress, node_layer:usize, path:&Path, child_index:usize) {
        let trail = self.get_trail(&root, path);
        let mut new_index = child_index;
        let steps = path.directions.len() - 1;
        for step in 0 ..= steps {
            let cur_address = NodeAddress::new(step + node_layer, trail[steps - step]);
            let child_direction = path.directions[steps - step] as usize; 
            new_index = self.add_modified_node(&cur_address, child_direction, new_index);
        }
        self.transfer_reference(&root, &NodeAddress::new(root.layer, new_index));
        root.index = new_index;
    }

}
