use macroquad::prelude::*;

mod graph;

mod game;
use game::*;


#[macroquad::main("Window")]
async fn main() {  
    let size = Vec2::new(512., 512.);
    request_new_screen_size(size.x, size.y);
    let mut scene = Scene::new();    
    let mut world = Object::new(scene.graph.get_root(), Vec2::new(size.x/2., size.y/2.), size.x);
    let mut player = Object::new(scene.graph.get_root(), Vec2::new(size.x/2., size.y/2.), 10.);

    let speed = 0.1;
    let torque = 0.05;
    let mut operation_depth = 0;
    let mut cur_color = MAROON;
    loop {

        if is_key_pressed(KeyCode::P) {
            scene.graph.profile();
        } else if is_key_pressed(KeyCode::V) {
            cur_color = match cur_color {
                BLACK => MAROON,
                MAROON => BLACK,
                _ => WHITE
            }
        }

        //Wow this is ugly, abstract this stuff away into a proper input handler?
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

        if is_key_down(KeyCode::A) {
            player.apply_rotational_force(-torque);
        }
        if is_key_down(KeyCode::D) {
            player.apply_rotational_force(torque);
        }
        if is_key_down(KeyCode::W) {
            player.apply_linear_force(Vec2::splat(speed));
        }
        if is_key_down(KeyCode::S) {
            player.apply_linear_force(-Vec2::splat(speed));
        }

        
        if is_mouse_button_down(MouseButton::Left) {
            scene.set_cell_with_mouse(&mut world, Vec2::from(mouse_position()), operation_depth, cur_color);
        }
       
        scene.render(&world, true);
        scene.render(&player, true);
        player.draw_facing();
        scene.move_with_collisions(&mut player, &world);
        next_frame().await
    }

}
