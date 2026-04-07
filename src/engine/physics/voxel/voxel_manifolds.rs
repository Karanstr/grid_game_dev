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
            camera.draw_vec_line(point, point + contact.dist * normal * -100., 4., GOLD);
        }
    }
}


#[derive(Clone, Copy)]
struct Contact {
    point: Vec2,
    depth: f32,
}
#[derive(Clone, Copy)]
struct ManifoldMeta {
    normal: Vec2,
    shape1: bool,
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
    for (contacts, meta) in unstructured_manifolds {
        let mut manifold = ContactManifold::new();
        if meta.shape1 {
            manifold.local_n1 = meta.normal;
            manifold.local_n2 = -pos21.transform_vector(manifold.local_n1);
            for contact in contacts {
                manifold.points.push(TrackedContact::new(
                    contact.point,
                    pos21.transform_point(contact.point),
                    PackedFeatureId::UNKNOWN,
                    PackedFeatureId::UNKNOWN,
                    -contact.depth
                ))
            }
        } else {
            manifold.local_n2 = meta.normal;
            manifold.local_n1 = -pos12.transform_vector(manifold.local_n2);
            for contact in contacts {
                manifold.points.push(TrackedContact::new(
                    pos12.transform_point(contact.point),
                    contact.point,
                    PackedFeatureId::UNKNOWN,
                    PackedFeatureId::UNKNOWN,
                    -contact.depth
                ))
            }
        }
        manifolds.push(manifold);
    }
}

// Written by AI
fn reduce_manifold_voxel_voxel(points: &[(Contact, ManifoldMeta)]) -> Vec<(Vec<Contact>, ManifoldMeta)> {
    let mut grouped: Vec<(Vec<Contact>, ManifoldMeta)> = Vec::new();

    'outer: for &(contact, meta) in points {
        // Check if this normal already exists
        for (contacts, m) in grouped.iter_mut() {
            if (m.normal - meta.normal).length_squared() < 1e-6 && meta.shape1 == m.shape1 {
                contacts.push(contact);
                continue 'outer;
            }
        }
        // New normal group
        grouped.push((vec![contact], meta));
    }

    // Reduce each group sharing a normal
    grouped.into_iter().map( |(contacts, meta)|
        (reduce_to_manifold(contacts, meta.normal), meta)
    ).collect()
}

fn reduce_to_manifold(contacts: Vec<Contact>, normal: Vec2) -> Vec<Contact> {
    if contacts.len() <= 1 { return contacts; }
    let tangent = normal.perp();

    // Deepest point (largest penetration)
    let deepest = contacts.iter()
        .max_by(|a, b| a.depth.total_cmp(&b.depth)).unwrap()
    ;

    // Furthest point along tangent from deepest
    let furthest = contacts.iter()
        .max_by(|a, b| {
            let da = (a.point - deepest.point).dot(tangent).abs();
            let db = (b.point - deepest.point).dot(tangent).abs();
            da.total_cmp(&db)
        }).unwrap()
    ;

    if (deepest.point - furthest.point).dot(tangent).abs() < f32::EPSILON {
        vec![*deepest]
    } else { vec![*deepest, *furthest] }
}

fn generate_contact_points_voxel_voxel(pos12: &Pose, shape1: &Voxels, shape2: &Voxels) -> Vec<(Contact, ManifoldMeta)> {
    let mut points = Vec::new();
    let pairs = dual_tree_descent(&pos12, shape1, shape2);

    let pose21 = pos12.inverse();
    for (idx1, idx2) in pairs.iter() {
        let (faces1, cell1) = &shape1.faces[*idx1];
        let (faces2, cell2) = &shape2.faces[*idx2];
        for (start1, end1, normal1) in generate_lines(faces1, cell1, shape1) {
            for (start2, end2, normal2) in generate_lines(faces2, cell2, shape2) {
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
                points.push( match (result1, result2) {
                    (Some(contact1), Some(contact2)) => {
                        if contact1.depth <= contact2.depth {
                            (contact1, ManifoldMeta{normal: normal1, shape1: true})
                        } else {
                            (contact2, ManifoldMeta{normal: normal2, shape1: false})
                        }
                    },
                    (Some(contact), None) => (
                        contact,
                        ManifoldMeta{normal: normal1, shape1: true}
                    ),
                    (None, Some(contact)) => (
                        contact,
                        ManifoldMeta{normal: normal2, shape1: false}
                    ),
                    (None, None) => continue,
                });
            }
        }
    }
    points
}

fn intersect_axis_aligned_with_depth(a1: Vec2, a2: Vec2, b1: Vec2, b2: Vec2, a_normal: Vec2) -> Option<Contact> {
    let (normal_idx, tangent_idx) = if a_normal.x != 0.0 { (0, 1) } else { (1, 0) };
    let b_dir = b2 - b1;
    let a_min = a1.min(a2);
    let a_max = a1.max(a2);

    let t = (a1[normal_idx] - b1[normal_idx]) / b_dir[normal_idx];
    if !(0.0..=1.0).contains(&t) { return None; }

    let point = b1 + b_dir * t;
    if !(a_min[tangent_idx] ..= a_max[tangent_idx]).contains(&point[tangent_idx]) { return None; }

    let pos_to_time = |pos: f32| ((pos - b1[tangent_idx]) / b_dir[tangent_idx]).clamp(0.0, 1.0);
    let depth_at  = |t: f32| a_normal[normal_idx] * (a1[normal_idx] - (b1 + b_dir * t)[normal_idx]);
    let (time_a, time_b) = (pos_to_time(a_min[tangent_idx]), pos_to_time(a_max[tangent_idx]));
    let depth = depth_at(time_a.min(time_b)).max(depth_at(time_a.max(time_b)));

    if depth > 0.0 { Some(Contact{point, depth}) } else { None }
}

/// Returns Vec<(start, end, normal)>
fn generate_lines(faces: &Faces, cell: &Cell, shape: &Voxels) -> Vec<(Vec2, Vec2, Vec2)> {
    let mut lines = Vec::new();
    for direction in faces.list() {
        // We do this silly extension thing to prevent squeezing between contacts caused by
        // the grid being chunked instead of a contiguous surface. If the face has a contiguous
        // face to a side, extend that face to that side some amount.
        // +-1 is an arbitrary amount which gets scaled, but I'm fairly sure we're fine.
        // If it breaks things, revisit this
        let (p1, p2) = match direction {
            Directions::North => {
                let (mut min, mut max) = (Vec2::ZERO, Vec2::new(1., 0.));
                if !faces.west() { min.x -= 1. }
                if !faces.east() { max.x += 1. }
                (min, max)
            },
            Directions::South => {
                let (mut min, mut max) = (Vec2::new(0., 1.), Vec2::ONE);
                if !faces.west() { min.x -= 1. }
                if !faces.east() { max.x += 1. }
                (min, max)
            },
            Directions::East => {
                let (mut min, mut max) = (Vec2::new(1., 0.), Vec2::ONE);
                if !faces.north() { min.y -= 1. }
                if !faces.south() { max.y += 1. }
                (min, max)
            },
            Directions::West => {
                let (mut min, mut max) = (Vec2::ZERO, Vec2::new(0., 1.));
                if !faces.north() { min.y -= 1. }
                if !faces.south() { max.y += 1. }
                (min, max)
            }
        };

        let (pos, size) = cell_pos_size(cell, shape.geometry.height);
        lines.push((
            pos + (p1 * size),
            pos + (p2 * size),
            direction.step().as_vec2()
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
