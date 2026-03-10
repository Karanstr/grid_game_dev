use glam::Vec2;
use rapier2d::math::Pose;

use crate::engine::{camera::Camera, grid::*};
use macroquad::color::*;

// I don't know where I want this yet
fn domain_length(height: u32) -> f32 {
    2u32.pow(height) as f32
}

impl super::Entity {
    pub fn draw(&self, graph: &SparseDirectedGraph<2, BasicNode2d>, pose: Pose, camera: &Camera, other_leaves: Vec<(Index, Vec<Zorder2d>)>) {
        let length = domain_length(self.geometry.height);
        let leaves = dfs_leaves(graph.nodes.unsafe_data(), self.geometry.head);
        render_leaves::<BasicNode2d>(leaves, self.geometry.height, Vec2::splat(-length / 2.), pose, camera);
        render_leaves::<BasicNode2d>(other_leaves, self.geometry.height, Vec2::splat(-length / 2.), pose, camera);
    }
}

pub fn render_leaves<N: Node<2>>(
    leaves: Vec<(Index, Vec<N::Children>)>,
    height: u32,
    origin: Vec2,
    pose: Pose,
    camera: &Camera,
) {
    for (idx, path) in leaves {
        let coords = N::Children::path_to_cell(&path);
        let cell_size = domain_length(height - path.len() as u32);
        let local_origin = Vec2::new(coords[0] as f32, coords[1] as f32) * cell_size + origin;
        let world_corners: [Vec2; 4] = [
            local_origin,
            local_origin + Vec2::new(cell_size, 0.0),
            local_origin + Vec2::new(cell_size, cell_size),
            local_origin + Vec2::new(0.0, cell_size),
        ].map(|point| pose * point );

        let color = match idx {
            0 => continue,
            1 => RED,
            2 => BLUE,
            3 => DARKGRAY,
            _ => unreachable!("Reached unregistered leaf {idx}")
        };
        camera.draw_rectangle_from_corners(&world_corners, color);
    }
}
