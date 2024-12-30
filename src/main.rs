mod engine;
use engine::graph::*;
use engine::game::*;
use engine::drawing_camera::Camera;
use engine::utilities::*;
use macroquad::prelude::*;
use std::fs;

#[macroquad::main("Window")]
async fn main() {
    let size = Vec2::splat(256.);
    request_new_screen_size(size.x*2., size.y*2.);
    let camera = Camera::new(AABB::new(size, size), Vec2::ZERO);
    let mut world = World::new(5, camera);
    let mut floor = Object::new(world.graph.get_root(0), size, 256., CollisionType::Static);
    //Loading
    {
        let save = fs::read_to_string("src/entities/world.json").unwrap();
        if !save.is_empty() {
            let new_root = world.graph.load_object_json(save);
            let old_root = floor.root;
            floor.root = new_root;
            world.graph.swap_root(old_root, new_root);
        }
    }
    world.add_object(floor);
    world.add_object(Object::new(world.graph.get_root(2), size, 32., CollisionType::Dynamic));
    let mut operation_depth = 0;
    let mut cur_block_index = 0;
    loop {
         
        //Profiling and save/load and zoom
        {
            if is_key_pressed(KeyCode::P) {
                world.graph.profile();
            } else if is_key_pressed(KeyCode::V) {
                cur_block_index = (cur_block_index + 1) % world.graph.leaf_count as usize;
            } else if is_key_pressed(KeyCode::K) {
                // let save = world.graph.save_object_json(floor.root);
                // let _ = fs::write("src/entities/world.json", save);
            } else if is_key_pressed(KeyCode::L) {
                let save = fs::read_to_string("src/entities/world.json").unwrap();
                let new_root = world.graph.load_object_json(save);
                let floor = world.access_object(0);
                let old_root = floor.root;
                floor.root = new_root;
                world.graph.swap_root(old_root, new_root);
            } else if is_key_pressed(KeyCode::Equal) {
                world.camera.zoom *= 1.1;
            } else if is_key_pressed(KeyCode::Minus) {
                world.camera.zoom /= 1.1;
            } else if is_key_pressed(KeyCode::F) {
                world.camera.shrink_view(Vec2::splat(200.));
            } else if is_key_pressed(KeyCode::G) {
                world.camera.expand_view(Vec2::splat(200.));
            } else if is_key_pressed(KeyCode::J) {
                world.expand_object_domain(0, 0);
            } else if is_key_pressed(KeyCode::H) {
                world.shrink_object_domain(0, 0);
            }

        } 

        //Depth changing
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
        
        //WASD Movement
        {
            let player = world.access_object(1);
            let speed = 0.2;
            if is_key_down(KeyCode::A) {
                player.apply_linear_force(Vec2::new(-speed, 0.));
            }
            if is_key_down(KeyCode::D) {
                player.apply_linear_force(Vec2::new(speed, 0.));
            }
            if is_key_pressed(KeyCode::Space) {
                player.apply_linear_force(Vec2::new(0., -6.));
            }
            player.apply_linear_force(Vec2::new(0., 0.22));
        }

        if is_mouse_button_down(MouseButton::Left) {
            if let Err(message) = world.set_cell_with_mouse(0, Vec2::from(mouse_position()), operation_depth, Index(cur_block_index)) {
                eprintln!("{message}");
            }
        }

        let new_cam_pos = world.access_object(1).aabb.center();
        world.camera.interpolate_position(new_cam_pos, 0.4);
        world.camera.show_view();
        world.draw_and_tick(true, true);
        next_frame().await
    }

}
