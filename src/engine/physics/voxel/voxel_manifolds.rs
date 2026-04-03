use std::cmp::Ordering;
use glam::Vec2;
use macroquad::color::GOLD;
use rapier2d::prelude::{BoundingVolume, PackedFeatureId, TrackedContact};
use rapier2d::{math::Pose, parry::bounding_volume::Aabb};
use rapier2d::parry::query::ContactManifold;
use crate::engine::camera::Camera;
use crate::engine::grid::dim2::*;
use crate::engine::physics::Faces;
use crate::engine::physics::voxel::voxel_faces::Directions;

use super::Voxels;

pub fn debug_voxel_voxel(
    pos12: &Pose,
    shape1_pos: &Pose,
    shape1: &Voxels,
    shape2: &Voxels,
    camera: &Camera
) {
    let mut manifolds = Vec::new();
    contact_manifold_voxel_voxel::<(), ()>(pos12, shape1, shape2, 0., &mut manifolds);
    for manifold in manifolds {
        let normal = shape1_pos.transform_vector(manifold.local_n1);
        for contact in manifold.points {
            let point = shape1_pos.transform_point(contact.local_p1);
            camera.draw_point(point, 0.1, GOLD);
            camera.draw_vec_line(point, point + contact.dist * normal * -1., 2., GOLD);
        }
    }
}

// Leverage subshapes/features using u64 zorder
pub fn contact_manifold_voxel_voxel<ManifoldData, ContactData>(
    pos12: &Pose,
    shape1: &Voxels,
    shape2: &Voxels,
    prediction: f32,
    manifolds: &mut Vec<ContactManifold<ManifoldData, ContactData>>
) where 
    ManifoldData: Default,
    ContactData: Default + Copy
{
    manifolds.clear();

    let points = generate_contact_points_voxel_voxel(&pos12, shape1, shape2);
    let unstructured_manifolds = reduce_manifold_voxel_voxel(&points);

    let pos21 = pos12.inverse();
    for ((normal, is_shape1), contacts) in unstructured_manifolds {
        let mut manifold = ContactManifold::new();
        if is_shape1 {
            manifold.local_n1 = normal;
            manifold.local_n2 = -pos21.transform_vector(manifold.local_n1);
            for (point, depth) in contacts {
                manifold.points.push(TrackedContact::new(
                    point,
                    pos21.transform_point(point),
                    PackedFeatureId::UNKNOWN,
                    PackedFeatureId::UNKNOWN,
                    -depth
                ))
            }
        } else {
            manifold.local_n2 = normal;
            manifold.local_n1 = -pos12.transform_vector(manifold.local_n2);
            for (point, depth) in contacts {
                manifold.points.push(TrackedContact::new(
                    pos12.transform_point(point),
                    point,
                    PackedFeatureId::UNKNOWN,
                    PackedFeatureId::UNKNOWN,
                    -depth
                ))
            }
        }
        manifolds.push(manifold);
    }
}

// Written by AI
fn reduce_manifold_voxel_voxel(points: &[(Vec2, f32, Vec2, bool)]) -> Vec<((Vec2, bool), Vec<(Vec2, f32)>)> {
    let mut grouped: Vec<((Vec2, bool), Vec<(Vec2, f32)>)> = Vec::new();

    'outer: for &(point, depth, normal, is_shape1) in points {
        // Check if this normal already exists
        for ((n, is), pts) in grouped.iter_mut() {
            if (*n - normal).length_squared() < 1e-6 && is_shape1 == *is {
                pts.push((point, depth));
                continue 'outer;
            }
        }
        // New normal group
        grouped.push(((normal, is_shape1), vec![(point, depth)]));
    }

    // Reduce each group along tangent
    grouped
        .into_iter()
        .map(|((normal, is_shape1), pts)| 
            ((normal, is_shape1), reduce_to_manifold(pts, normal))
        ).collect()
}

/// returns Vec<(point, depth, is_shape1)>
fn reduce_to_manifold(points: Vec<(Vec2, f32)>, normal: Vec2) -> Vec<(Vec2, f32)> {
    if points.len() <= 1 { return points; }
    let tangent = normal.perp();

    // Deepest point (largest penetration)
    let deepest = points.iter()
        .max_by(|a, b| a.1.total_cmp(&b.1)).unwrap()
    ;

    // Furthest point along tangent from deepest
    let furthest = points.iter()
        .max_by(|a, b| {
            let da = (a.0 - deepest.0).dot(tangent).abs();
            let db = (b.0 - deepest.0).dot(tangent).abs();
            da.total_cmp(&db)
        }).unwrap()
    ;

    if (deepest.0 - furthest.0).dot(tangent).abs() < f32::EPSILON {
        vec![*deepest]
    } else { vec![*deepest, *furthest] }
}

/// Returns (point, depth, normal, is_shape1)
fn generate_contact_points_voxel_voxel(pos12: &Pose, shape1: &Voxels, shape2: &Voxels) -> Vec<(Vec2, f32, Vec2, bool)> {
    let mut points = Vec::new();
    let pairs = dual_tree_descent(&pos12, shape1, shape2);

    let pose21 = pos12.inverse();
    for (idx1, idx2) in pairs.iter() {
        let (faces1, cell1) = &shape1.faces[*idx1];
        let (faces2, cell2) = &shape2.faces[*idx2];
        for (start2, end2, normal2) in generate_lines(faces2, cell2, shape2) {
            for (start1, end1, normal1) in generate_lines(faces1, cell1, shape1) {
                let result1 = intersect_axis_aligned_with_depth(
                    start1,
                    end1,
                    pos12.transform_point(start2),
                    pos12.transform_point(end2),
                    normal1
                );
                let result2 = intersect_axis_aligned_with_depth(
                    start2,
                    end2,
                    pose21.transform_point(start1),
                    pose21.transform_point(end1),
                    normal2
                );
                let result = match (result1, result2) {
                    (Some(res1), Some(res2)) => {
                        if res1.1 <= res2.1 { 
                            Some((res1.0, res1.1, res1.2, true))
                        } else {
                            Some((res2.0, res2.1, res2.2, false))
                        }
                    },
                    (Some(res), None) => Some((res.0, res.1, res.2, true)),
                    (None, Some(res)) => Some((res.0, res.1, res.2, false)),
                    (None, None) => None,
                };
                let Some(result) = result else { continue };

                points.push(result);
            }
        }
    }
    points
}

// Written by AI
fn intersect_axis_aligned_with_depth(
    a1: Vec2, a2: Vec2,
    b1: Vec2, b2: Vec2,
    axis_aligned_direction: Directions,
) -> Option<(Vec2, f32, Vec2)> {
    let normal = axis_aligned_direction.step().as_vec2();
    let ax = if normal.x != 0.0 { 0 } else { 1 }; // normal axis
    let tg = 1 - ax;                              // tangent axis
    let b_dir = b2 - b1;
    let a_min = a1.min(a2);
    let a_max = a1.max(a2);

    let t = (a1[ax] - b1[ax]) / b_dir[ax];
    if !(0.0..=1.0).contains(&t) { return None; }

    let p = b1 + b_dir * t;
    if p[tg] < a_min[tg] || p[tg] > a_max[tg] { return None; }

    let pos_to_time = |val: f32| ((val - b1[tg]) / b_dir[tg]).clamp(0.0, 1.0);
    let (ta, tb) = (pos_to_time(a_min[tg]), pos_to_time(a_max[tg]));
    let depth_at  = |s: f32| normal[ax] * (a1[ax] - (b1 + b_dir * s)[ax]);
    let depth = depth_at(ta.min(tb)).max(depth_at(ta.max(tb)));

    if depth > 0.0 { Some((p, depth, normal)) } else { None }
}

/// Returns Vec<(start, end, normal)>
fn generate_lines(faces: &Faces, cell: &Cell, shape: &Voxels) -> Vec<(Vec2, Vec2, Directions)> {
    let mut lines = Vec::new();
    for direction in faces.list() {
        let (p1, p2) = match direction {
            Directions::North => (Vec2::ZERO, Vec2::new(1., 0.)),
            Directions::South => (Vec2::new(0., 1.), Vec2::ONE),
            Directions::East => (Vec2::new(1., 0.), Vec2::ONE),
            Directions::West => (Vec2::ZERO, Vec2::new(0., 1.))
        };
        let (pos, size) = cell_pos_size(cell, shape.geometry.height);
        lines.push((
            pos + (p1 * size),
            pos + (p2 * size),
            direction
        ));
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
    pos12: &Pose,
    shape1: &Voxels,
    shape2: &Voxels,
) -> Vec<(usize, usize)> {
    if shape1.faces.is_empty() || shape2.faces.is_empty() { 
        eprintln!("Empty Shape passed to dual_tree_descent!!!");
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
            .translated(pos1)
        ;
        let (pos2, size2) = cell_pos_size(&node2.cell, shape2.geometry.height);
        let aabb2 = Aabb::new(Vec2::ZERO, Vec2::splat(size2))
            .translated(pos2).transform_by(&pos12)
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
