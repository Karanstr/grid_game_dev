use glam::Vec2;
use rapier2d::math::Pose;

use crate::engine::{camera::Camera, grid::*, physics::Voxels};
use macroquad::color::*;

impl super::Entity {
    pub fn draw(&self, pose: Pose, camera: &Camera, cells: &[(Index, Cell)], opacity: f32) {
        let length = Voxels::length(self.geometry.height);
        let tl_pose = pose.prepend_translation(Vec2::splat(-length / 2.));
        render_leaves(&cells, self.geometry.height, tl_pose, camera, opacity);
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
