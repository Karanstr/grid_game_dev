mod engine;
use imports::*;
mod imports {
    use super::*;
    pub use engine::physics::collisions::{n_body_collisions, Corners, corner_handling::*};
    pub use engine::grid::dag::{ExternalPointer, Index};
    pub use macroquad::math::{Vec2, BVec2, IVec2, Mat2};
    pub use engine::grid::partition::*;
    pub use engine::entities::*;
    pub use macroquad::input::*;
    pub use derive_new::new;
    pub use engine::math::*;
    pub use super::Location;
    pub use std::f32::consts::PI;
}
mod globals {
    use crate::engine::blocks::BlockPalette;
    use crate::engine::grid::dag::{SparseDirectedGraph, BasicNode};
    use crate::engine::camera::Camera;
    use macroquad::math::Vec2;
    use crate::engine::math::Aabb;
    use crate::engine::entities::EntityPool;
    use lazy_static::lazy_static;
    use parking_lot::RwLock;
    lazy_static! { 
        pub static ref GRAPH: RwLock<SparseDirectedGraph<BasicNode>> = RwLock::new(SparseDirectedGraph::<BasicNode>::new(4));
        pub static ref ENTITIES: RwLock<EntityPool> = RwLock::new(EntityPool::new());
        pub static ref CAMERA: RwLock<Camera> = RwLock::new(Camera::new(
            Aabb::new(Vec2::ZERO, Vec2::splat(8.)), 
            0.9
        ));
        pub static ref BLOCKS: BlockPalette = BlockPalette::default();
    }
}
use globals::*;
use engine::input::*;

use std::time::Duration;
use std::thread;
fn init_deadlock_detection() {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let deadlocks = parking_lot::deadlock::check_deadlock();
            if !deadlocks.is_empty() {
                println!("{} deadlocks detected", deadlocks.len());
                for deadlock in deadlocks {
                    println!("Deadlock threads:");
                    for thread in deadlock {
                        println!("Thread Id {:#?}", thread.thread_id());
                        println!("Backtrace:\n{:#?}", thread.backtrace());
                    }
                }
                panic!("Deadlock detected");
            }
        }
    });
}

use lazy_static::lazy_static;
use parking_lot::RwLock;
#[derive(new)]
pub struct InputData {
    pub movement_id : ID,
    pub edit_id : ID,
    pub edit_color : usize,
    pub edit_height : u32,
}
lazy_static! {
    pub static ref INPUT_DATA: RwLock<InputData> = RwLock::new(InputData::new(0, 0, 0, 0));
}

const PLAYER_SPEED: f32 = 0.01;
const PLAYER_ROTATION_SPEED: f32 = PI/256.;
const PLAYER_SPAWN: Vec2 = Vec2::new(0.,0.);
const TERRAIN_SPAWN: Vec2 = Vec2::new(0.,0.);
const PLAYER_ROTATION_SPAWN: f32 = 0.;
const TERRAIN_ROTATION_SPAWN: f32 = 0.;
const MAX_COLOR: usize = 4;
const MAX_HEIGHT: u32 = 4;

#[derive(Debug, Clone, Copy, new)]
pub struct Location {
    pub position: Vec2,
    pub pointer: ExternalPointer,
    #[new(value = "Vec2::splat(1.0)")]
    pub min_cell_length: Vec2,
}
impl Location {
    pub fn to_aabb(&self) -> Aabb {
        Aabb::new(self.position, center_to_edge(self.pointer.height, self.min_cell_length))
    }
}

fn set_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        println!("Panic: {:?}", panic_info);
    }));
}

fn mouse_pos() -> Vec2 { Vec2::from(mouse_position()) }

#[macroquad::main("Window")]
async fn main() {
    set_panic_hook();
    init_deadlock_detection();
    #[cfg(debug_assertions)]
    println!("Debug mode");
    #[cfg(not(debug_assertions))]
    println!("Release mode");
    macroquad::window::request_new_screen_size(1024., 1024.);
    
    let terrain_id = {
        let world_pointer = {
            let string = std::fs::read_to_string("data/save.json").unwrap_or_default();
            if string.is_empty() { 
                GRAPH.write().get_root(0, 3)
            } else { 
                GRAPH.write().load_object_json(string)
            }
        };
        ENTITIES.write().add_entity(
            Location::new(TERRAIN_SPAWN, world_pointer),
            TERRAIN_ROTATION_SPAWN,
        )
    };

    let player_id = {
        //Graph has to be unlocked before add_entity is called so corners can be read from it
        let player_pointer = GRAPH.write().get_root(3, 0);
        ENTITIES.write().add_entity(
            Location::new(PLAYER_SPAWN, player_pointer),
            PLAYER_ROTATION_SPAWN,
        )
    };

    INPUT_DATA.write().edit_id = terrain_id;
    INPUT_DATA.write().movement_id = player_id;

    let mut input = set_key_binds();
    
    loop {

        input.handle();
        ENTITIES.read().draw_all(true);
        
        // We want to move the camera to where the player is drawn, not where the player is moved to.
        let player_pos = ENTITIES.read().get_entity(player_id).unwrap().location.position;
        
        n_body_collisions(terrain_id).await;
        
        // We don't want to move the camera until after we've drawn all the collision debug.
        // This ensures everything lines up with the current frame.
        CAMERA.write().update(player_pos, 1.);
        
        macroquad::window::next_frame().await
    }

}

pub fn set_grid_cell(entity:ID, world_point:Vec2, new_cell:ExternalPointer) {
    let mut entities = ENTITIES.write();
    let entity = &mut entities.get_mut_entity(entity).unwrap();
    if new_cell.height > entity.location.pointer.height { return; }
    let Some(cell) = gate::point_to_cells(entity.location, new_cell.height, world_point)[0] else { return };
    let path = ZorderPath::from_cell(cell, entity.location.pointer.height - new_cell.height);
    let Ok(root) = GRAPH.write().set_node(entity.location.pointer, &path.steps(), new_cell.pointer) else {
        dbg!("Failed to set cell");
        return;
    };
    entity.set_root(root);
}

pub fn set_key_binds() -> InputHandler {
    let mut input = InputHandler::new();
    // Movement
    input.bind_key(KeyCode::W, InputTrigger::Down, move || {
        let id = INPUT_DATA.read().movement_id;
        ENTITIES.write().get_mut_entity(id).unwrap().apply_abs_velocity(Vec2::new(0., -PLAYER_SPEED));
    });
    input.bind_key(KeyCode::S, InputTrigger::Down, move || {
        let id = INPUT_DATA.read().movement_id;
        ENTITIES.write().get_mut_entity(id).unwrap().apply_abs_velocity(Vec2::new(0., PLAYER_SPEED));
    });
    input.bind_key(KeyCode::A, InputTrigger::Down, move || {
        let id = INPUT_DATA.read().movement_id;
        ENTITIES.write().get_mut_entity(id).unwrap().apply_abs_velocity(Vec2::new(-PLAYER_SPEED, 0.));
    });
    input.bind_key(KeyCode::D, InputTrigger::Down, move || {
        let id = INPUT_DATA.read().movement_id;
        ENTITIES.write().get_mut_entity(id).unwrap().apply_abs_velocity(Vec2::new(PLAYER_SPEED, 0.));
    });
    input.bind_key(KeyCode::Q, InputTrigger::Down, move || {
        let id = INPUT_DATA.read().movement_id;
        ENTITIES.write().get_mut_entity(id).unwrap().angular_velocity -= PLAYER_ROTATION_SPEED;
    });
    input.bind_key(KeyCode::E, InputTrigger::Down, move || {
        let id = INPUT_DATA.read().movement_id;
        ENTITIES.write().get_mut_entity(id).unwrap().angular_velocity += PLAYER_ROTATION_SPEED;
    });
    input.bind_key(KeyCode::Space, InputTrigger::Down, move || {
        let id = INPUT_DATA.read().movement_id;
        ENTITIES.write().get_mut_entity(id).unwrap().velocity = Vec2::ZERO;
        ENTITIES.write().get_mut_entity(id).unwrap().angular_velocity = 0.0;
    });

    // Editing
    input.bind_key(KeyCode::V, InputTrigger::Pressed, move || {
        let color = &mut INPUT_DATA.write().edit_color;
        *color = (*color + 1) % MAX_COLOR;
    });
    input.bind_key(KeyCode::B, InputTrigger::Pressed, move || {
        let height = &mut INPUT_DATA.write().edit_height;
        *height = (*height + 1) % MAX_HEIGHT;
    });
    input.bind_mouse(MouseButton::Left, InputTrigger::Down, move || {
        let data = INPUT_DATA.read();
        set_grid_cell(
            data.edit_id,
            CAMERA.read().screen_to_world(mouse_pos()),
            ExternalPointer::new(Index(data.edit_color), data.edit_height)
        );
    });
    
    // Save/Load
    input.bind_key(KeyCode::K, InputTrigger::Pressed, move || {
        let id = INPUT_DATA.read().edit_id;
        let save_data = GRAPH.read().save_object_json(ENTITIES.read().get_entity(id).unwrap().location.pointer);
        std::fs::write("data/save.json", save_data).unwrap();
    });
    input.bind_key(KeyCode::L, InputTrigger::Pressed, move || {
        let id = INPUT_DATA.read().edit_id;
        let mut entities = ENTITIES.write();
        let terrain_entity = entities.get_mut_entity(id).unwrap();
        let new_pointer = {
            let mut graph = GRAPH.write();
            let save_data = std::fs::read_to_string("data/save.json").unwrap();
            let new_pointer = graph.load_object_json(save_data);
            new_pointer
        };
        terrain_entity.location.pointer = new_pointer;
    });

    // Debug
    input.bind_key(KeyCode::P, InputTrigger::Pressed, move || {
        dbg!(GRAPH.read().nodes.internal_memory());
    });

    input
}
