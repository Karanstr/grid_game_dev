//For now only implementing 1 dimension

#[derive(Debug)]
pub struct Node {
    pub child_indexes:[usize; 2],
    pub ref_count:u8,
}

impl Node {
    fn new_empty() -> Self {
        Self {
            child_indexes: [0, 0],
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

//Write get_child_address
impl NodeAddress {
    pub fn new(layer:usize, index:usize) -> Self {
        Self {
            layer,
            index
        }
    }

}

pub struct SparseDAG1D {
    pub node_pot:Vec<Vec<Node>>
}

impl SparseDAG1D {

    pub fn new(max_layer:usize) -> Self {
        let mut new_self = Self {
            node_pot : Vec::new()
        };
        new_self.ensure_depth(max_layer);
        new_self
    }

    //Public methods used to meta-edit the dag
    //Rename these stupid variable names
    pub fn compress_root_once(&mut self, root:&mut NodeAddress) {
        let cur_node = self.get_node(root);
        let (kid_count, last_index) = cur_node.count_kids_and_get_last();
        if  kid_count == 1 {
            self.lower_root_by_one(root, last_index);
        } else {
            let kids = cur_node.child_indexes;
            let child_directions = [1, 0];
            let mut new_root_node = Node::new_empty();
            for kid in 0 .. kids.len() {
                let kid_kids = self.get_node(&NodeAddress::new(root.layer - 1, kids[kid]));
                let (kid_kid_count, kid_index) = kid_kids.count_kids_and_get_last();
                if !(kid_kid_count == 1 && kid_index == child_directions[kid]) || kid_kid_count == 0 {
                    return
                }
                new_root_node.child_indexes[kid] = kid_kids.child_indexes[child_directions[kid]]
            } //If we don't return, we are good to continue
            let mut new_root = NodeAddress::new(root.layer - 1, 0);
            new_root.index = self.add_node(root.layer - 1, new_root_node);
            self.transfer_reference(&root, &new_root);
            root.layer = new_root.layer;
            root.index = new_root.index;
        }
    }


    //Private methods used to meta-edit the dag 
    fn ensure_depth(&mut self, layer:usize) {
        for _i in self.node_pot.len()..=layer {
            self.node_pot.push(vec![Node::new_empty()]);
        }
    }

    //Combine raise and lower, remove the by_one, make these both private
    //Raise_root_by_one doesn't delete references either, get rid of the set_node_child call while merging and implement how lower_root does it
    //Mutates root
    pub fn raise_root_by_one(&mut self, root:&mut NodeAddress, direction:u32) {
        //Arbitrary limit to prevent overflows in df_to_binary
        if root.layer == 6-1 {
            return
        }
        self.ensure_depth(root.layer + 1);
        let mut new_root = NodeAddress::new(root.layer + 1, 0);
        self.set_node_child(&mut new_root, root.layer + 1, direction, root.index);
        root.layer = new_root.layer;
        root.index = new_root.index;
    }

    pub fn lower_root_by_one(&mut self, root:&mut NodeAddress, direction:usize) {
        if root.layer == 0 { return }
        let child_index = self.get_node_child_index(&root, direction);
        self.transfer_reference(&root, &NodeAddress::new(root.layer - 1, child_index));
        root.layer = root.layer - 1;
        root.index = child_index;
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

    fn get_trail(&self, root:&NodeAddress, path:u32, steps:usize) -> Vec<usize> {
        let mut trail:Vec<usize> = Vec::with_capacity(steps + 1);
        trail.push(root.index); //We start our journey at the root
        for step in 0..=steps {
            let cur_address= NodeAddress::new(root.layer - step, trail[step]);
            let child_direction:usize = ((path >> (steps - step)) & 0b1) as usize;
            trail.push(self.get_node_child_index(&cur_address, child_direction));
        }
        trail
    }


    //Public methods used to read from the dag
    pub fn get_node(&self, address:&NodeAddress) -> &Node {
        &self.node_pot[address.layer][address.index]
    }

    pub fn read_node_child(&self, root:&NodeAddress, node_layer:usize, path:u32) -> usize {
        let steps = root.layer - node_layer;
        let trail = self.get_trail(&root, path, steps);
        let address = NodeAddress::new(node_layer, trail[steps]);
        self.get_node_child_index(&address, path as usize & 0b1)
    }

    //Works with cell_counts of up to 64. More than that and the u64 overflows. Solution in the works. (probably rewrite my packed_array)
    pub fn df_to_binary(&self, root:&NodeAddress) -> u64 {
        let mut resulting_binary:u64 = 0;
        let mut queue: Vec<(NodeAddress, u32)> = Vec::new();
        queue.push((root.clone(), 0));

        while queue.len() != 0 {
            let (cur_address, cur_path) = queue.pop().unwrap();
            let cur_node = self.get_node(&cur_address);
            for child in 0..2 {
                let child_index = cur_node.child_indexes[child];
                let child_path = (cur_path << 1) | child as u32;
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
        if address.index != 0 {
            queue.push(address.clone());
        }

        while queue.len() != 0 {
            let cur_address = queue.pop().unwrap();
            let cur_node = self.get_mut_node(&cur_address);
            cur_node.ref_count -= 1;
            //If node needs to be deleted
            if cur_node.ref_count == 0 {
                //If has children nodes
                if cur_address.layer != 0 {
                    for child in cur_node.child_indexes.iter() {
                        //If child isn't a null node
                        if *child != 0 {
                            queue.push(NodeAddress::new(cur_address.layer - 1, *child));
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

    //Must consume node, as node_pot needs ownership of its nodes
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

    fn transfer_reference(&mut self, node_give:&NodeAddress, node_recieve:&NodeAddress) {
        self.inc_ref_count(node_recieve);
        self.dec_ref_count(node_give);
    }


    //Public methods used to modify dag data
    //Modifies root to point right to the new root.
    pub fn set_node_child(&mut self, root:&mut NodeAddress, node_layer:usize, path:u32, child_index:usize) {
        let steps = root.layer - node_layer;
        let trail = self.get_trail(&root, path, steps);
        let mut new_index = child_index;
        for step in 0..=steps {
            let cur_address:NodeAddress = NodeAddress::new(step + node_layer, trail[steps - step]);
            let child_direction:usize = ((path >> step) & 0b1) as usize; 
            new_index = self.add_modified_node(&cur_address, child_direction, new_index);
        }
        self.transfer_reference(&root, &NodeAddress::new(root.layer, new_index));
        root.index = new_index;
    }



}
