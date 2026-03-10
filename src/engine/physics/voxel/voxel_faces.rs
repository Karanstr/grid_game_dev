use glam::UVec2;
use crate::engine::grid::*;


impl super::Voxels {
    // Sets new shape geometry, forcing internal regeneration of exposed faces
    pub fn update_shape(&mut self, geometry: DagPointer) -> Vec<(u32, Vec<Zorder2d>)> {
        self.geometry = geometry;
        self.cache_faces()
    }

    // Based on dfs_leaves algorithm :(
    fn cache_faces(&mut self) -> Vec<(u32, Vec<Zorder2d>)>{
        let graph = self.graph.read();

        let mut stack = vec![(self.geometry.head as usize, Vec::new())];
        let mut leaves = Vec::new();
    
        let nodes = graph.nodes.unsafe_data();
        'search: while let Some((idx, zorder)) = stack.pop() {
            // We don't track air nodes
            // Hack until I have a proper block attribute system
            if idx == 0 { continue 'search }
            let cur_node = nodes[idx];
            for child in Zorder2d::all().iter().rev() {
                let child_idx = cur_node.get(*child);
                // This is stupid and deceptive, doing this nonsense *inside* of the loop.
                // The current alternative is stupider though..
                if child_idx == idx as u32 {
                    leaves.extend(identify_faces(&graph, self.geometry.head, &zorder));
                    // Prevents other iterations of the children to run, because the parent is a leaf..
                    continue 'search
                }
                let mut child_zorder = zorder.clone();
                child_zorder.push(*child);
                stack.push((child_idx as usize, child_zorder));
            }
        }

        let real_leaves = leaves.into_iter().map(|(_, zorder)| {
            (3, zorder)
        }).collect();
        real_leaves

    }
}

fn identify_faces(
    graph: &SparseDirectedGraph<2, BasicNode2d>,
    head: u32,
    path: &Vec<Zorder2d>,
) -> Vec<(Faces, Vec<Zorder2d>)> {
    let center = UVec2::from_array(Zorder2d::path_to_cell(path));
    let mut results = Vec::new();

    let mut faces = Faces::new();

    // Checking [2, 3]
    // Splits [0, 1]
    if let Some(north_cell) = center.checked_sub(UVec2::Y) {
        let north_path = Zorder2d::path_from_cell(
            north_cell.to_array(),
            path.len() as u32
        ).unwrap();
        let result = graph.descend(head, &north_path);
        if result == 0 {
            // If block is air, do nothing as faces intializes assuming all is true.
        } else if result < 4 {
            // If block is a leaf (Hack until I write attributes)
            faces.north = false;
        } else {
            // If block isn't a leaf
            faces.north = false;
            let mut left_new_path = path.clone();
            left_new_path.push(Zorder2d::new([0, 0]).unwrap());
            results.extend(identify_faces(graph, head, &left_new_path));
            
            let mut right_new_path = path.clone();
            right_new_path.push(Zorder2d::new([1, 0]).unwrap());
            results.extend(identify_faces(graph, head, &right_new_path));
        }

    }

    // Checking [0, 1]
    // Splits [2, 3]
    let south_cell = center + UVec2::Y;
    if south_cell.max_element() < 2u32.pow(path.len() as u32)  {

        let south_path = Zorder2d::path_from_cell(
            south_cell.to_array(),
            path.len() as u32
        ).unwrap();
        let result = graph.descend(head, &south_path);
        if result == 0 {
            // If block is air, do nothing as faces intializes assuming all is true.
        } else if result < 4 {
            // If block is a leaf (Hack until I write attributes)
            faces.south = false;
        } else {
            // If block isn't a leaf
            faces.south = false;
            let mut left_new_path = path.clone();
            left_new_path.push(Zorder2d::new([0, 1]).unwrap());
            results.extend(identify_faces(graph, head, &left_new_path));
            
            let mut right_new_path = path.clone();
            right_new_path.push(Zorder2d::new([1, 1]).unwrap());
            results.extend(identify_faces(graph, head, &right_new_path));
        }
    }

    // Checking [1, 3]
    // Splits [0, 2]
    if let Some(west_cell) = center.checked_sub(UVec2::X) {
        let west_path = Zorder2d::path_from_cell(
            west_cell.to_array(),
            path.len() as u32
        ).unwrap();
        let result = graph.descend(head, &west_path);
        if result == 0 {
            // If block is air, do nothing as faces intializes assuming all is true.
        } else if result < 4 {
            // If block is a leaf (Hack until I write attributes)
            faces.west = false;
        } else {
            // If block isn't a leaf
            faces.west = false;
            let mut left_new_path = path.clone();
            left_new_path.push(Zorder2d::new([0, 0]).unwrap());
            results.extend(identify_faces(graph, head, &left_new_path));
            
            let mut right_new_path = path.clone();
            right_new_path.push(Zorder2d::new([0, 1]).unwrap());
            results.extend(identify_faces(graph, head, &right_new_path));
        }

    }

    // Checking [0, 2]
    // Splits [1, 3]
    let east_cell = center + UVec2::X;
    if east_cell.max_element() < 2u32.pow(path.len() as u32)  {

        let east_path = Zorder2d::path_from_cell(
            east_cell.to_array(),
            path.len() as u32
        ).unwrap();
        let result = graph.descend(head, &east_path);
        if result == 0 {
            // If block is air, do nothing as faces intializes assuming all is true.
        } else if result < 4 {
            // If block is a leaf (Hack until I write attributes)
            faces.east = false;
        } else {
            // If block isn't a leaf
            faces.east = false;
            let mut left_new_path = path.clone();
            left_new_path.push(Zorder2d::new([1, 0]).unwrap());
            results.extend(identify_faces(graph, head, &left_new_path));
            
            let mut right_new_path = path.clone();
            right_new_path.push(Zorder2d::new([1, 1]).unwrap());
            results.extend(identify_faces(graph, head, &right_new_path));
        }
    }

    if faces.has_exposed_face() {
        results.insert(0, (faces, path.clone()));
    }
    results
}

struct Faces {
    north: bool,
    east: bool,
    south: bool,
    west: bool
}
impl Faces {
    fn new() -> Self {
        Self {
            north: true,
            east: true,
            south: true,
            west: true,
        }
    }
    fn has_exposed_face(&self) -> bool {
        self.north || self.east || self.south || self.west
    }
}

// Write condensed Zorder converter
pub struct FaceNode {
    node: DagPointer,
    location: Vec<Zorder2d>,
    faces: Faces,
}

