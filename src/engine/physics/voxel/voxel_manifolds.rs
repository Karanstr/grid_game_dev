use std::cmp::Ordering;

use glam::Vec2;
use rapier2d::prelude::BoundingVolume;
use rapier2d::{math::Pose, parry::bounding_volume::Aabb};
use rapier2d::parry::query::ContactManifold;
use crate::engine::camera::Camera;
use crate::engine::grid::dim2::*;

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

pub struct PotentialPair {
    pub cell1: Cell,
    pub face1: usize,
    pub cell2: Cell,
    pub face2: usize,
}

// Warn: Currently assumes shape1 and shape2 are stored within the same DAG. I can't imagine a case where
// this wouldn't be true, but worth noting down
pub fn dual_tree_descent(
    pos12: &Pose,
    shape1: &Voxels,
    shape2: &Voxels,
    camera: &Camera,
) -> Vec<PotentialPair> {
    if shape1.faces.is_empty() || shape2.faces.is_empty() { 
        dbg!("Empty Shape passed to dual_tree_descent!!!");
        return vec![];
    }
    let mut candidates = Vec::new();
    let mut stack = Vec::new();

    let root_face1 = if let Some((_, cell)) = shape1.faces.get(0) { 
        if cell.len() == 0 { Some(0) } else { None }
    } else { None };
    let root1 = Descent::new( Cell::default(), shape1.geometry, root_face1);

    let root_face2 = if let Some((_, cell)) = shape2.faces.get(0) { 
        if cell.len() == 0 { Some(0) } else { None }
    } else { None };
    let root2 = Descent::new(Cell::default(), shape2.geometry, root_face2);

    let topleft_offset = Vec2::splat(-Voxels::length(shape1.geometry.height) / 2.);
    let tl_pos12 = pos12.prepend_translation(topleft_offset);
    stack.push((root1, root2));

    while let Some((tree1, tree2)) = stack.pop() {

        let length1 = Vec2::splat(Voxels::length(tree1.pointer.height));
        let aabb1 = Aabb::new(Vec2::ZERO, length1)
            .translated(get_cell_origin(&tree1.cell, shape1.geometry.height) + topleft_offset)
        ;
        let length2 = Vec2::splat(Voxels::length(tree2.pointer.height));
        let aabb2 = Aabb::new(Vec2::ZERO, length2)
            .translated(get_cell_origin(&tree2.cell, shape2.geometry.height))
            .transform_by(&tl_pos12)
        ;

        if !aabb1.intersects(&aabb2) { continue }
        camera.outline_aabb(&aabb2);
        
        match (tree1.face, tree2.face) {
            (Some(face1), Some(face2)) => {
                let pair = PotentialPair {
                    cell1: tree1.cell.clone(),
                    face1,
                    cell2: tree2.cell.clone(),
                    face2,
                };
                candidates.push(pair);
            }
            (None, Some(_)) => {
                let splits = descend_split(&tree1, shape1);
                for split in splits.into_iter().flatten() {
                    stack.push((split, tree2.clone()));
                }
            }
            (None, None) if tree1.pointer.height >= tree2.pointer.height => {
                let splits = descend_split(&tree1, shape1);
                for split in splits.into_iter().flatten() {
                    stack.push((split, tree2.clone()));
                }
            }
            (Some(_), None) => {
                let splits = descend_split(&tree2, shape2);
                for split in splits.into_iter().flatten() {
                    stack.push((tree1.clone(), split));
                }
            }
            (None, None) if tree1.pointer.height < tree2.pointer.height => {
                let splits = descend_split(&tree2, shape2);
                for split in splits.into_iter().flatten() {
                    stack.push((tree1.clone(), split));
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

fn get_cell_origin(cell: &Cell, head_height: u32) -> Vec2 {
    let coords = cell.cell();
    let cell_size = Voxels::length(head_height - cell.len() as u32);
    Vec2::new(coords[0] as f32, coords[1] as f32) * cell_size
}
