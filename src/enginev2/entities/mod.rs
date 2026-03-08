use crate::enginev2::{camera::Camera, grid::*, physics::*};
use glam::Vec2;
use lilypads::Pond;
use macroquad::color::{BLUE, DARKGRAY, RED};
use rapier2d::{math::Pose, prelude::{ColliderBuilder, ColliderHandle, RigidBodyHandle, SharedShape}};

// https://docs.rs/hecs/latest/hecs/

fn domain_length(height: u32) -> f32 {
    2u32.pow(height) as f32
}

pub struct EntityPool {
    entities: Pond<Entity>,
    pub graph: crate::GRAPH,
}
impl EntityPool {

    pub fn new(graph: crate::GRAPH) -> Self {
        Self {
            entities: Pond::new(),
            graph,
        }
    }

    pub fn add(&mut self, geometry: DagPointer, position: Vec2, physics: &mut Physics) -> usize {
        let collider = ColliderBuilder::new(SharedShape::new(
            Voxels::new(geometry, self.graph.clone())
        ))
            .translation(position)
            .rotation(-0.5);
        let collider_handle = physics.colliders.insert(collider);
        let entity = Entity {
            rb_handle: None,
            collider_handle,
            geometry,
        };
        self.entities.insert(entity)
    }

    pub fn draw_all(&self, physics: &Physics, camera: &Camera) {
        for (_, entity) in self.entities.iter() {
            let pose = *physics.colliders
                .get(entity.collider_handle).unwrap()
                .position()
            ;
            entity.draw(&self.graph.read(), pose, camera);
        }
    }
}

pub struct Entity {
    rb_handle: Option<RigidBodyHandle>,
    collider_handle: ColliderHandle,

    geometry: DagPointer,
}
impl Entity {
    
    pub fn set_geometry(&mut self, geometry: DagPointer, physics: &mut Physics) {
        self.geometry = geometry;

        let shape = physics.colliders
            .get_mut(self.collider_handle).unwrap()
            .shape_mut()
            .downcast_mut::<Voxels>().unwrap()
        ;
        shape.geometry = geometry;
    }

    pub fn draw(&self, graph: &SparseDirectedGraph<2, BasicNode2d>, pose: Pose, camera: &Camera) {
        let length = domain_length(self.geometry.height);
        render_leaves(
            graph.nodes.unsafe_data(),
            self.geometry.head,
            length,
            Vec2::splat(-length / 2.),
            pose,
            camera
        );
    }
}

fn render_leaves<N: Node<2>>(
    nodes: &Vec<N>,
    head: Index,
    cell_size: f32,
    origin: Vec2,
    pose: Pose,
    camera: &Camera,
) {
    let node = nodes[head as usize];
    
    for &child in N::Children::all() {
        let child_idx = node.get(child);
        
        if child_idx == head {
            let world_corners: [Vec2; 4] = [
                origin,
                origin + Vec2::new(cell_size, 0.0),
                origin + Vec2::new(cell_size, cell_size),
                origin + Vec2::new(0.0, cell_size),
            ].map(|point| pose * point );
            let color = match head {
                0 => return,
                1 => RED,
                2 => BLUE,
                3 => DARKGRAY,
                _ => panic!("Reached unregistered leaf {head}")
            };
            camera.draw_rectangle_from_corners(&world_corners, color);
            return;
        } else {
            let subcell = child.subcell();
            let child_size = cell_size / 2.0;
            let child_origin = origin + Vec2::new(
                subcell[0] as f32 * child_size,
                subcell[1] as f32 * child_size,
            );
            render_leaves::<N>(nodes, child_idx, child_size, child_origin, pose, camera);
        }
    }
}
