mod graph;
mod game;
mod drawing_camera;
mod utilities;

use std::f32::consts::PI;
use graph::*;
use game::*;
use drawing_camera::Camera;
use utilities::*;
use macroquad::prelude::*;

#[macroquad::main("Window")]
async fn main() {
    let size = Vec2::new(512., 512.);
    request_new_screen_size(size.x, size.y);
    let camera = Camera::new(AABB::new(size/2., size/2.), Vec2::ZERO);
    let mut world = World::new(5, camera);
    let mut block = Object::new(world.graph.get_root(1), size/2. + 100., 32.);
    let mut player = Object::new(world.graph.get_root(4), size/2., 32.);
    let mut static_block = Object::new(world.graph.get_root(0), size/2., 256.);
    let speed = 0.2;
    let torque = 0.08;
    let mut operation_depth = 0;
    let mut cur_block_index = 0;
    let mut save = world.graph.save_object_json(block.root);
    loop {
        
        //Profiling and player speed-reorientation and save/load and zoom
        {
            if is_key_pressed(KeyCode::P) {
                world.graph.profile();
            } else if is_key_pressed(KeyCode::V) {
                cur_block_index = (cur_block_index + 1) % 5
            } else if is_key_pressed(KeyCode::H) {
                player.set_rotation(0.);
            } else if is_key_pressed(KeyCode::R) {
                player.set_rotation(PI/2.);
            } else if is_key_pressed(KeyCode::T) {
                player.set_rotation(PI/4.);
            } else if is_key_pressed(KeyCode::K) {
                save = world.graph.save_object_json(block.root);
            } else if is_key_pressed(KeyCode::L) {
                let new_root = world.graph.load_object_json(save.clone());
                let old_root = block.root;
                block.root = new_root;
                world.graph.swap_root(old_root, new_root);
            } else if is_key_pressed(KeyCode::Equal) {
                world.camera.zoom *= 1.1;
            } else if is_key_pressed(KeyCode::Minus) {
                world.camera.zoom /= 1.1;
            } else if is_key_pressed(KeyCode::F) {
                world.camera.shrink_view(Vec2::splat(200.));
            } else if is_key_pressed(KeyCode::G) {
                world.camera.expand_view(Vec2::splat(200.));
            }
        } 

        //Handle depth changing
        {
            if is_key_pressed(KeyCode::Key1) {
                operation_depth = 1;
            } else if is_key_pressed(KeyCode::Key2) {
                operation_depth = 2;
            } else if is_key_pressed(KeyCode::Key3) {
                operation_depth = 3;
            } else if is_key_pressed(KeyCode::Key4) {
                operation_depth = 4;
            } else if is_key_pressed(KeyCode::Key5) {
                operation_depth = 5;
            } else if is_key_pressed(KeyCode::Key6) {
                operation_depth = 6;
            } else if is_key_pressed(KeyCode::Key7) {
                operation_depth = 7;
            } else if is_key_pressed(KeyCode::Key8) {
                operation_depth = 8;
            } else if is_key_pressed(KeyCode::Key9) {
                operation_depth = 9;
            } else if is_key_pressed(KeyCode::Key0) {
                operation_depth = 0;
            }
        }
        
        //WASD Tank Movement
        {
            if is_key_down(KeyCode::A) {
                player.apply_rotational_force(-torque);
            }
            if is_key_down(KeyCode::D) {
                player.apply_rotational_force(torque);
            }
            if is_key_down(KeyCode::W) {
                player.apply_forward_force(Vec2::splat(speed));
            }
            if is_key_down(KeyCode::S) {
                player.apply_forward_force(-Vec2::splat(speed));
            }
            if is_key_down(KeyCode::Space) {
                player.velocity = Vec2::ZERO;
            }
        }

        if is_mouse_button_down(MouseButton::Left) {
            if let Err(message) = world.set_cell_with_mouse(&mut static_block, Vec2::from(mouse_position()), operation_depth, Index(cur_block_index)) {
                eprintln!("{message}");
            }
        }

        
        world.camera.interpolate_position(player.aabb.center(), 0.4);
        world.camera.outline_bounds(world.camera.aabb, 2., WHITE);
        // world.render(&mut block, true);
        world.render(&mut player, true);
        world.render(&mut static_block, true);
        player.draw_facing(&world.camera);
        world.n_body_collisions(Vec::from([&mut player, /*&mut block,*/ &mut static_block]), 10.);

        draw_text(&format!("{:.0}", player.aabb.center()), 10., 10., 20., WHITE);
        next_frame().await
    }

}
