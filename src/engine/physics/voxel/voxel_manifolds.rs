use std::cmp::Ordering;
use glam::Vec2;
use rapier2d::prelude::BoundingVolume;
use rapier2d::{math::Pose, parry::bounding_volume::Aabb};
use rapier2d::parry::query::ContactManifold;
use crate::engine::grid::dim2::*;
use crate::engine::physics::Faces;
use crate::engine::physics::voxel::voxel_faces::Directions;

use super::Voxels;

#[allow(unused_variables)]
pub fn contact_manifold_voxel_voxel<ManifoldData, ContactData>(
    pos12: &Pose,
    shape1: &Voxels,
    shape2: &Voxels,
    prediction: f32,
    manifolds: &mut Vec<ContactManifold<ManifoldData, ContactData>>
) {

}

// Returns [manifold_points, all_points]
pub fn contact_debug_voxel_voxel(
    pos12: &Pose,
    shape1: &Voxels,
    shape2: &Voxels,
) -> [Vec<(Vec2, f32, Vec2)>; 2] {
    let points = generate_contact_points_voxel_voxel(pos12, shape1, shape2);
    let mut manifold_points = Vec::new();
    let mut north = Vec::new();
    let mut south = Vec::new();
    let mut east = Vec::new();
    let mut west = Vec::new();

    for (point, depth, normal) in points.clone() {
        match normal {
            Vec2::NEG_Y => north.push((point, depth, normal)),
            Vec2::Y => south.push((point, depth, normal)),
            Vec2::X => east.push((point, depth, normal)),
            Vec2::NEG_X => west.push((point, depth, normal)),
            _ => {}
        }
    }
    manifold_points.extend(reduce_to_manifold(north));
    manifold_points.extend(reduce_to_manifold(south));
    manifold_points.extend(reduce_to_manifold(east));
    manifold_points.extend(reduce_to_manifold(west));
    
    [manifold_points, points]
}

// Also written by ai for now
fn reduce_to_manifold(points: Vec<(Vec2, f32, Vec2)>) -> Vec<(Vec2, f32, Vec2)> {
    if points.len() <= 1 { return points; }

    // Deepest point
    let deepest = points.iter()
        .copied()
        .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();

    // Point furthest from the deepest
    let furthest = points.iter()
        .copied()
        .max_by(|a, b| {
            a.0.distance_squared(deepest.0)
                .partial_cmp(&b.0.distance_squared(deepest.0))
                .unwrap()
        }).unwrap();

    if deepest.0.distance_squared(furthest.0) < f32::EPSILON {
        vec![deepest]
    } else {
        vec![deepest, furthest]
    }
}

/// Returns (point, depth, normal)
fn generate_contact_points_voxel_voxel(
    pos12: &Pose,
    shape1: &Voxels,
    shape2: &Voxels,
) -> Vec<(Vec2, f32, Vec2)> {
    let mut points = Vec::new();
    let topleft_offset = Vec2::splat(-Voxels::length(shape1.geometry.height) / 2.);
    let tl_pos12 = pos12.prepend_translation(topleft_offset);
    let pairs = dual_tree_descent(topleft_offset, &tl_pos12, shape1, shape2);

    for (idx1, idx2) in pairs.iter() {
        let (faces1, cell1) = &shape1.faces[*idx1];
        let (faces2, cell2) = &shape2.faces[*idx2];
        let lines1 = generate_lines(faces1, cell1, shape1);
        let lines2 = generate_lines(faces2, cell2, shape2);
        for [start2, end2, _] in lines2.iter() {
            let start2 = tl_pos12.transform_point(*start2);
            let end2 = tl_pos12.transform_point(*end2);
            for [start1, end1, normal] in lines1.iter() {
                if let Some((point, depth)) = intersect_axis_aligned_with_depth(
                    start1 + topleft_offset,
                    end1 + topleft_offset,
                    start2,
                    end2,
                    *normal
                ) { points.push((point, depth, *normal)) }
            }
        }
    }
    points
}

// Written by AI, don't trust
fn intersect_axis_aligned_with_depth(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2, normal: Vec2) -> Option<(Vec2, f32)> {
    let b_dir = b2 - b1;
    let axis = if normal.x != 0.0 { 0 } else { 1 };
    let other = 1 - axis;
    let d = b_dir[axis];
    if d.abs() < f32::EPSILON { return None; }
    let t = (a1[axis] - b1[axis]) / d;
    if !(0.0..=1.0).contains(&t) { return None; }
    let p = b1 + b_dir * t;
    let a_min = a1.min(a2);
    let a_max = a1.max(a2);
    if p[other] < a_min[other] || p[other] > a_max[other] { return None; }

    // Find t values where B crosses the face extents on the other axis,
    // clamped to the valid segment range [0, 1]
    let t_at_other = |val: f32| -> f32 {
        if b_dir[other].abs() < f32::EPSILON { t } else { (val - b1[other]) / b_dir[other] }
    };

    let ta = t_at_other(a_min[other]).clamp(0.0, 1.0);
    let tb = t_at_other(a_max[other]).clamp(0.0, 1.0);
    let (t_lo, t_hi) = (ta.min(tb), ta.max(tb));

    // Depth is measured purely along the normal axis — parallel segments
    // can only be as deep as their actual normal-axis distance to the face
    let depth_at = |s: f32| normal[axis] * (a1[axis] - (b1 + b_dir * s)[axis]);
    let depth = depth_at(t_lo).max(depth_at(t_hi));
    // Some((p, depth))
    if depth > 0.0 { Some((p, depth)) } else { None }
}

/// Returns ([start, end, normal])
fn generate_lines(faces: &Faces, cell: &Cell, shape: &Voxels) -> Vec<[Vec2; 3]> {
    let mut lines = Vec::new();
    for direction in faces.list() {
        let (p1, p2, normal) = match direction {
            Directions::North => (Vec2::ZERO, Vec2::new(1., 0.), Vec2::NEG_Y),
            Directions::South => (Vec2::new(0., 1.), Vec2::ONE, Vec2::Y),
            Directions::East => (Vec2::new(1., 0.), Vec2::ONE, Vec2::X),
            Directions::West => (Vec2::ZERO, Vec2::new(0., 1.), Vec2::NEG_X)
        };
        // I know not exactly optimal
        let (pos, size) = cell_pos_size(cell, shape.geometry.height);
        lines.push([
            pos + (p1 * size),
            pos + (p2 * size),
            normal
        ]);
    }
    lines
}

#[derive(Clone, Debug)]
struct Descent {
    cell: Cell,
    pointer: DagPointer,
    face: Option<usize>,
}
impl Descent {
    fn new(cell: Cell, pointer: DagPointer, face: Option<usize>) -> Self {
        Self {
            cell,
            pointer,
            face,
        }
    }
}

// Warn: Currently assumes shape1 and shape2 are stored within the same DAG. 
// I can't imagine a case where this wouldn't be true, but worth noting down
/// Returns all colliding pairs (a, b), where a indexes shape1.faces and b indexes shape2.faces
fn dual_tree_descent(
    topleft_offset: Vec2,
    tl_pos12: &Pose,
    shape1: &Voxels,
    shape2: &Voxels,
) -> Vec<(usize, usize)> {
    if shape1.faces.is_empty() || shape2.faces.is_empty() { 
        dbg!("Empty Shape passed to dual_tree_descent!!!");
        return vec![];
    }
    let mut candidates = Vec::new();
    let mut stack = Vec::new();

    let root1 = Descent::new(
        Cell::default(), shape1.geometry,
        if let Some((_, cell)) = shape1.faces.get(0) && cell.len() == 0 { Some(0) } else { None }
    );
    let root2 = Descent::new(
        Cell::default(), shape2.geometry,
        if let Some((_, cell)) = shape2.faces.get(0) && cell.len() == 0 { Some(0) } else { None }
    );
    stack.push((root1, root2));

    while let Some((node1, node2)) = stack.pop() {

        let (pos1, size1) = cell_pos_size(&node1.cell, shape1.geometry.height);
        let aabb1 = Aabb::new(Vec2::ZERO, Vec2::splat(size1))
            .translated(pos1 + topleft_offset)
        ;
        let (pos2, size2) = cell_pos_size(&node2.cell, shape2.geometry.height);
        let aabb2 = Aabb::new(Vec2::ZERO, Vec2::splat(size2))
            .translated(pos2).transform_by(&tl_pos12)
        ;
        if !aabb1.intersects(&aabb2) { continue }
        
        match (node1.face, node2.face) {
            (Some(face1), Some(face2)) => {
                candidates.push((face1, face2));
            }
            (None, Some(_)) => {
                let splits = descend_split(&node1, shape1);
                for split in splits.into_iter().flatten() {
                    stack.push((split, node2.clone()));
                }
            }
            (None, None) if node1.pointer.height >= node2.pointer.height => {
                let splits = descend_split(&node1, shape1);
                for split in splits.into_iter().flatten() {
                    stack.push((split, node2.clone()));
                }
            }
            (Some(_), None) => {
                let splits = descend_split(&node2, shape2);
                for split in splits.into_iter().flatten() {
                    stack.push((node1.clone(), split));
                }
            }
            (None, None) if node1.pointer.height < node2.pointer.height => {
                let splits = descend_split(&node2, shape2);
                for split in splits.into_iter().flatten() {
                    stack.push((node1.clone(), split));
                }
            }
            _ => unreachable!()
        }
        
    }

    candidates
}

fn descend_split(split_descent: &Descent, split_shape: &Voxels) -> [Option<Descent>; 4] {
    let mut results = [None, None, None, None];
    let graph = split_shape.graph.read();
    let node = graph.node(split_descent.pointer.head);
    for &child in Zorder2d::all().iter().rev() {
        let mut new_cell = split_descent.cell.clone();
        new_cell.push_step(child);

        let Ok(face) = split_shape.faces.binary_search_by(|(_, cell)| ancestry(cell, &new_cell)) else {
            continue
        };
        // If the cell matched another cell on its real level, we've reached a leaf
        let face = if new_cell.len() == split_shape.faces[face].1.len() { 
            Some(face)
        } else { None };

        // Warn: This is technically not safe, if the pointer height is malformed then this can underflow!!
        let new_pointer = DagPointer::new(node.child(child), split_descent.pointer.height - 1);
        results[child.as_usize()] = Some(Descent::new(new_cell, new_pointer, face));
    }
    results
}

/// Equal means one is a subcell of the other
fn ancestry(cell1: &Cell, cell2: &Cell) -> Ordering {
    let mut packed1 = PackedCell::default();
    let mut packed2 = PackedCell::default();
    for i in 0 .. cell1.len().min(cell2.len()) {
        packed1.push_step(cell1.step_at(i).unwrap());
        packed2.push_step(cell2.step_at(i).unwrap());
    }
    packed1.packed().cmp(&packed2.packed())
}

fn cell_pos_size(cell: &Cell, head_height: u32) -> (Vec2, f32) {
    let coords = cell.cell();
    let cell_size = Voxels::length(head_height - cell.len() as u32);
    (Vec2::new(coords[0] as f32, coords[1] as f32) * cell_size, cell_size)
}
