use glam::Vec2;
use rapier2d::math::Pose;

use crate::engine::{camera::Camera, grid::*, physics::Faces};
use macroquad::color::*;

// I don't know where I want this yet
fn domain_length(height: u32) -> f32 {
    2u32.pow(height) as f32
}

impl super::Entity {
    pub fn draw(&self, graph: &SparseDirectedGraph<2, BasicNode2d>, pose: Pose, camera: &Camera, other_leaves: &Vec<(Faces, Vec<Zorder2d>)>) {
        let length = domain_length(self.geometry.height);
        let leaves = dfs_leaves(graph.nodes.unsafe_data(), self.geometry.head);
        render_leaves::<BasicNode2d>(&leaves, self.geometry.height, Vec2::splat(-length / 2.), pose, camera);
        render_faces(other_leaves, self.geometry.height, Vec2::splat(-length / 2.), pose, camera);
    }
}

fn render_leaves<N: Node<2>>(
    leaves: &Vec<(Index, Vec<N::Children>)>,
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

// Could be better but I don't care
fn render_faces(
    leaves: &Vec<(Faces, Vec<Zorder2d>)>,
    height: u32,
    origin: Vec2,
    pose: Pose,
    camera: &Camera,
) {
    for (faces, path) in leaves {
        let coords = Zorder2d::path_to_cell(&path);
        let cell_size = domain_length(height - path.len() as u32);
        let local_origin = Vec2::new(coords[0] as f32, coords[1] as f32) * cell_size + origin;
        let world_corners: [Vec2; 4] = [
            local_origin,
            local_origin + Vec2::new(cell_size, 0.0),
            local_origin + Vec2::new(cell_size, cell_size),
            local_origin + Vec2::new(0.0, cell_size),
        ].map(|point| pose * point );
        
        if faces.north() {
            let point1 = world_corners[0];
            let point2 = world_corners[1];
            camera.draw_vec_line(point1, point2, 2., GREEN);
        }
        if faces.south() {
            let point1 = world_corners[2];
            let point2 = world_corners[3];
            camera.draw_vec_line(point1, point2, 2., GREEN);
        }
        if faces.east() {
            let point1 = world_corners[1];
            let point2 = world_corners[2];
            camera.draw_vec_line(point1, point2, 2., GREEN);
        }
        if faces.west() {
            let point1 = world_corners[0];
            let point2 = world_corners[3];
            camera.draw_vec_line(point1, point2, 2., GREEN);
        }
    }
}
