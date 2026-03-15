use glam::Vec2;
use rapier2d::math::Pose;

use crate::engine::{camera::Camera, grid::*, physics::{Faces, Voxels}};
use macroquad::color::*;

impl super::Entity {
    pub fn draw(&self, pose: Pose, camera: &Camera, cells: &[(Index, Cell)], opacity: f32) {
        let length = Voxels::length(self.geometry.height);
        let tl_pose = pose.prepend_translation(Vec2::splat(-length / 2.));
        render_leaves(&cells, self.geometry.height, tl_pose, camera, opacity);
    }

    pub fn outline(&self, pose: Pose, camera: &Camera, faces: &[(Faces, Cell)]) {
        let length = Voxels::length(self.geometry.height);
        let tl_pose = pose.prepend_translation(Vec2::splat(-length / 2.));
        render_faces(&faces, self.geometry.height, tl_pose, camera);
    }
}

pub fn render_leaves(
    leaves: &[(Index, Cell)],
    height: u32,
    tl_pose: Pose,
    camera: &Camera,
    opacity: f32,
) {
    for (idx, path) in leaves {
        let coords = path.cell();
        let cell_size = Voxels::length(height - path.len() as u32);
        let local_origin = Vec2::new(coords[0] as f32, coords[1] as f32) * cell_size;
        let world_corners: [Vec2; 4] = [
            local_origin,
            local_origin + Vec2::new(cell_size, 0.0),
            local_origin + Vec2::new(cell_size, cell_size),
            local_origin + Vec2::new(0.0, cell_size),
        ].map(|point| tl_pose * point );

        let color = match idx {
            0 => continue,
            1 => RED,
            2 => BLUE,
            3 => DARKGRAY,
            _ => unreachable!("Reached unregistered leaf {idx}")
        }.with_alpha(opacity);
        camera.draw_rectangle_from_corners(&world_corners, color);
    }
}

// Could be better but I don't care
fn render_faces(
    leaves: &[(Faces, Cell)],
    height: u32,
    tl_pose: Pose,
    camera: &Camera,
) {
    for (faces, path) in leaves {
        let coords = path.cell();
        let cell_size = Voxels::length(height - path.len() as u32);
        let local_origin = Vec2::new(coords[0] as f32, coords[1] as f32) * cell_size;
        let world_corners: [Vec2; 4] = [
            local_origin,
            local_origin + Vec2::new(cell_size, 0.0),
            local_origin + Vec2::new(cell_size, cell_size),
            local_origin + Vec2::new(0.0, cell_size),
        ].map(|point| tl_pose * point );
        
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
