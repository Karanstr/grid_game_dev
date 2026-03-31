use glam::{IVec2, UVec2};
use crate::engine::grid::*;


impl super::Voxels {
    // Sets new shape geometry, forcing internal regeneration of exposed faces
    pub fn update_shape(&mut self, geometry: DagPointer) {
        self.geometry = geometry;
        self.faces = self.cache_faces();
    }

    // Based on dfs_leaves algorithm :(
    fn cache_faces(&mut self) -> Vec<(Faces, Cell)> {
        let graph = self.graph.read();

        let mut stack = vec![(self.geometry.head as usize, Cell::default())];
        let mut leaves = Vec::new();
    
        let nodes = graph.nodes.unsafe_data();
        while let Some((idx, path)) = stack.pop() {
            let cur_node = nodes[idx];
            // Hack until I have a proper block attribute system
            if cur_node.child(Zorder2d::TopLeft) == idx as u32 {
                // We don't track air nodes, though they are a leaf
                if idx != 0 { leaves.extend(identify_faces(&graph, self.geometry.head, &path, &Directions::all())) }
                continue
            }
            for &child in Zorder2d::all().iter().rev() {
                let child_idx = cur_node.child(child);
                let mut child_path = path.clone();
                child_path.push_step(child);
                stack.push((child_idx as usize, child_path));
            }
        }

        leaves

    }
}

// At some point cache the descent so when we recurse we don't perform a full tree descension again..
fn identify_faces(
    graph: &SparseDirectedGraph<2, BasicNode2d>,
    head: u32,
    path: &Cell,
    faces_to_check: &[Directions],
) -> Vec<(Faces, Cell)> {
    let mut results = Vec::new();
    let mut exposed = Faces::none();
    let mut splits = Faces::none();
    
    let max_bound = 2i32.pow(path.len() as u32);
    let center = UVec2::from_array(path.cell()).as_ivec2();

    for direction in faces_to_check {
        let idx = *direction as usize;
        let check_cell = center + direction.step();
        if check_cell.min_element() < 0 || check_cell.max_element() >= max_bound { 
            exposed.0[idx] = true;
            continue;
        }
    
        let check_path = Cell::new(check_cell.as_uvec2().to_array(), path.len());
        let result = graph.descend(head, &check_path);
        if result == 0 { exposed.0[idx] = true; } else if result >= 4 { splits.0[idx] = true; }
    }
    
    // If no splits we're done here
    if !splits.has_some() {
        if exposed.has_some() { results.push( (exposed, path.clone()) ) }
        return results
    }

    const CORNERS: [(Zorder2d, Directions, Directions); 4] = [
        (Zorder2d::TopLeft,     Directions::North, Directions::West),
        (Zorder2d::TopRight,    Directions::North, Directions::East),
        (Zorder2d::BottomLeft,  Directions::South, Directions::West),
        (Zorder2d::BottomRight, Directions::South, Directions::East),
    ];
    let mut child_path = path.clone();
    for (corner, dir_a, dir_b) in CORNERS {
        child_path.push_step(corner);
        results.extend(identify_faces(graph, head, &child_path, &[dir_a, dir_b]));
        child_path.pop_step();
    }
    
    results
}

/// [North, South, East, West]
#[derive(Clone, Debug)]
pub struct Faces([bool; 4]);
impl Faces {
    fn none() -> Self { Self([false; 4]) }
    pub fn north(&self) -> bool { self.0[0] }
    pub fn south(&self) -> bool { self.0[1] }
    pub fn east(&self) -> bool { self.0[2] }
    pub fn west(&self) -> bool { self.0[3] }
    pub fn has_some(&self) -> bool { self.0[0] || self.0[1] || self.0[2] || self.0[3] }
    // This is dumb, but I care about it working rn
    pub fn list(&self) -> Vec<Directions> {
        let mut list = Vec::new();
        if self.north() { list.push(Directions::North) }
        if self.south() { list.push(Directions::South) }
        if self.east() { list.push(Directions::East) }
        if self.west() { list.push(Directions::West) }
        list
    }
}
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Directions {
    North,
    South,
    East,
    West,
}
impl Directions {
    fn all() -> [Self; 4] {
        [Self::North, Self::South, Self::East, Self::West]
    }
    pub fn step(&self) -> IVec2 {
        match self {
            Self::North => IVec2::new(0, -1),
            Self::South => IVec2::new(0, 1),
            Self::East  => IVec2::new(1, 0), 
            Self::West  => IVec2::new(-1, 0), 
        }
    }
}

