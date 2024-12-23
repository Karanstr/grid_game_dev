use std::f32::consts::PI;
use macroquad::prelude::*;
use macroquad::miniquad::window::screen_size;
mod graph;

mod game;
use game::*;

#[macroquad::main("Window")]
async fn main() {
    let size = Vec2::new(512., 512.);
    request_new_screen_size(size.x+200., size.y+200.);
    let camera = game::Camera::new(size/2., Vec2::ZERO, size + 200.);
    let mut world = World::new(5, camera);
    let mut block = Object::new(world.graph.get_root(1), size/2. + 100., 32.);
    let mut player = Object::new(world.graph.get_root(4), size/2., 32.);
    let speed = 0.2;
    let torque = 0.08;
    let mut operation_depth = 0;
    let mut cur_block_index = 0;
    let mut save = world.graph.save_object_json(block.root);
    loop {

        //Profiling and player speed-reorientation and save/load
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
        }

        if is_mouse_button_down(MouseButton::Left) {
            if let Err(message) = world.set_cell_with_mouse(&mut block, Vec2::from(mouse_position()), operation_depth, Index(cur_block_index)) {
                eprintln!("{message}");
            }
        }
        if is_mouse_button_down(MouseButton::Right) {
            if let Err(message) = world.set_cell_with_mouse(&mut player, Vec2::from(mouse_position()), operation_depth, Index(cur_block_index)) {
                eprintln!("{message}");
            }
        }


        world.render(&mut block, true);
        world.render(&mut player, true);
        player.draw_facing(&world.camera);
        world.render_cache();
        // world.render_corners(&player, 5);
        // world.render_corners(&fixed, 5);
        world.two_way_collisions(&mut player, &mut block, 10.);
        world.camera.update(player.aabs.center, Vec2::from(screen_size()));
        world.camera.interpolate_offset(player.velocity*5., 0.1 );

        draw_text(&format!("{:.0}", player.aabs.center), 10., 10., 20., WHITE);
        next_frame().await
    }

}
