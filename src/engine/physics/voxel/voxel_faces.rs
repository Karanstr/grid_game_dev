use glam::{IVec2, UVec2};
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
                    leaves.extend(identify_faces(&graph, self.geometry.head, &zorder, &Directions::all()));
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

// At some point cache the descent so when we recurse we don't perform a full tree descension again..
fn identify_faces(
    graph: &SparseDirectedGraph<2, BasicNode2d>,
    head: u32,
    path: &Vec<Zorder2d>,
    faces_to_check: &[Directions],
) -> Vec<(Faces, Vec<Zorder2d>)> {
    let mut results: Vec<(Faces, Vec<Zorder2d>)> = Vec::new();
    let mut exposed = Faces::none();
    let mut splits = Faces::none();
    
    let max_bound = 2i32.pow(path.len() as u32);
    let center = UVec2::from_array(Zorder2d::path_to_cell(path)).as_ivec2();

    for direction in faces_to_check {
        let idx = *direction as usize;
        let check_cell = center + direction.step();
        if check_cell.min_element() < 0 || check_cell.max_element() >= max_bound { 
            exposed.0[idx] = true;
            continue;
        }
    
        let check_path = Zorder2d::path_from_cell(
            check_cell.as_uvec2().to_array(),
            path.len() as u32
        ).unwrap();
        let result = graph.descend(head, &check_path);
        if result == 0 { exposed.0[idx] = true; } else if result >= 4 { splits.0[idx] = true; }
    }

    // Perform splits without duplication
    let mut child_path = path.clone();
    if splits.north() || splits.west() {
        child_path.push(Zorder2d::TopLeft);
        let directions: &[Directions] = 
            if splits.north() && splits.west() { &[Directions::North, Directions::West] }
            else if splits.north() { &[Directions::North] }
            else { &[Directions::West] }
        ;
        results.extend(identify_faces(graph, head, &child_path, directions));
        child_path.pop();
    }
    if splits.north() || splits.east() {
        child_path.push(Zorder2d::TopRight);
        let directions: &[Directions] = 
            if splits.north() && splits.east() { &[Directions::North, Directions::East] }
            else if splits.north() { &[Directions::North] }
            else { &[Directions::East] }
        ;
        results.extend(identify_faces(graph, head, &child_path, directions));
        child_path.pop();
    }
    if splits.south() || splits.west() {
        child_path.push(Zorder2d::BottomLeft);
        let directions: &[Directions] = 
            if splits.south() && splits.west() { &[Directions::South, Directions::West] }
            else if splits.south() { &[Directions::South] }
            else { &[Directions::West] }
        ;
        results.extend(identify_faces(graph, head, &child_path, directions));
        child_path.pop();
    }
    if splits.south() || splits.east() {
        child_path.push(Zorder2d::BottomRight);
        let directions: &[Directions] =
            if splits.south() && splits.east() { &[Directions::South, Directions::East] }
            else if splits.south() { &[Directions::South] }
            else { &[Directions::East] }
        ;
        results.extend(identify_faces(graph, head, &child_path, directions));
        child_path.pop();
    }

    if exposed.has_some() { results.insert(0, (exposed, path.clone()));}
    results
}

// [North, South, East, West]
struct Faces([bool; 4]);
impl Faces {
    fn none() -> Self { Self([false; 4]) }
    fn north(&self) -> bool { self.0[0] }
    fn south(&self) -> bool { self.0[1] }
    fn east(&self) -> bool { self.0[2] }
    fn west(&self) -> bool { self.0[3] }
    fn has_some(&self) -> bool { self.0[0] || self.0[1] || self.0[2] || self.0[3] }
}
#[derive(Clone, Copy)]
#[repr(u8)]
enum Directions {
    North,
    South,
    East,
    West,
}
impl Directions {
    fn all() -> [Self; 4] {
        [Self::North, Self::South, Self::East, Self::West]
    }
    fn step(&self) -> IVec2 {
        match self {
            Self::North => IVec2::new(0, -1),
            Self::South => IVec2::new(0, 1),
            Self::East  => IVec2::new(1, 0), 
            Self::West  => IVec2::new(-1, 0), 
        }
    }
}

// Write condensed Zorder converter
pub struct FaceNode {
    node: DagPointer,
    location: Vec<Zorder2d>,
    faces: Faces,
}

