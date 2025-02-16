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

pub trait DataAccess {
    fn target_id(&self) -> ID;
    fn edit_color(&self) -> usize;
    fn edit_height(&self) -> u32;
    fn file_paths(&self) -> &[String; 2];
}
pub struct InputData {
    pub target_id : ID,
    pub edit_color : usize,
    pub edit_height : u32,
    pub file_paths : [String; 2],
}
impl Default for InputData {
    fn default() -> Self {
        Self {
            target_id: 1,
            edit_color: 0,
            edit_height: 0,
            file_paths: ["data/terrain.json".to_string(), "data/player.json".to_string()],
        }
    }
}
impl DataAccess for InputData {
    fn target_id(&self) -> ID { self.target_id }
    fn edit_color(&self) -> usize { self.edit_color }
    fn edit_height(&self) -> u32 { self.edit_height }
    fn file_paths(&self) -> &[String; 2] { &self.file_paths }
}


const SPEED: f32 = 0.01;
const ROTATION_SPEED: f32 = PI/256.;
const MAX_COLOR: usize = 4;
const MAX_HEIGHT: u32 = 4;

use serde::{Serialize, Deserialize};
// Move location into entity
#[derive(Debug, Clone, Copy, new, Serialize, Deserialize)]
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
    if !cfg!(target_arch = "wasm32") {
        set_panic_hook();
        init_deadlock_detection();
    }
    #[cfg(debug_assertions)]
    println!("Debug mode");
    #[cfg(not(debug_assertions))]
    println!("Release mode");
    macroquad::window::request_new_screen_size(1024., 1024.);
    {
        let mut entity_pool = ENTITIES.write();
        // Wasm Compatability
        // let string = if cfg!(target_arch = "wasm32") { 
        //     String::from_utf8(include_bytes!("../data/save.json").as_ref().to_vec()).unwrap_or_default()
        // }
        entity_pool.add_to_pool(
            Entity::load(std::fs::read_to_string("data/terrain.json").unwrap_or_default(), 0)
        );
        entity_pool.add_to_pool(
            Entity::load(std::fs::read_to_string("data/player.json").unwrap_or_default(), 1)
        );
    }
    
    let mut vars = InputData::default();
    let mut input = set_key_binds();
    
    loop {

        let old_pos = {
            let entities = ENTITIES.read();
            entities.draw_all(true);
            let target = entities.get_entity(vars.target_id()).unwrap();
            target.draw_outline(macroquad::color::DARKBLUE);
            
            // We want to move the camera to where the target is drawn, not where the target is moved to.
            target.location.position
        };
        
        input.handle(&mut vars);
        
        n_body_collisions((vars.target_id() + 1) % 2).await;
        
        // We don't want to move the camera until after we've drawn all the collision debug.
        // This ensures everything lines up with the current frame.
        CAMERA.write().update(old_pos, 0.4);
        
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

pub fn set_key_binds() -> InputHandler<InputData> {
    let mut input = InputHandler::new();
    // Movement
    input.bind_key(KeyCode::W, InputTrigger::Down, |data : &mut InputData| {
        let id = data.target_id();
        ENTITIES.write().get_mut_entity(id).unwrap().apply_abs_velocity(Vec2::new(0., -SPEED));
    });
    input.bind_key(KeyCode::S, InputTrigger::Down, |data : &mut InputData| {
        let id = data.target_id();
        ENTITIES.write().get_mut_entity(id).unwrap().apply_abs_velocity(Vec2::new(0., SPEED));
    });
    input.bind_key(KeyCode::A, InputTrigger::Down, |data : &mut InputData| {
        let id = data.target_id();
        ENTITIES.write().get_mut_entity(id).unwrap().apply_abs_velocity(Vec2::new(-SPEED, 0.));
    });
    input.bind_key(KeyCode::D, InputTrigger::Down, |data : &mut InputData| {
        let id = data.target_id();
        ENTITIES.write().get_mut_entity(id).unwrap().apply_abs_velocity(Vec2::new(SPEED, 0.));
    });
    input.bind_key(KeyCode::Q, InputTrigger::Down, |data : &mut InputData| {
        let id = data.target_id();
        ENTITIES.write().get_mut_entity(id).unwrap().angular_velocity -= ROTATION_SPEED;
    });
    input.bind_key(KeyCode::E, InputTrigger::Down, |data : &mut InputData| {
        let id = data.target_id();
        ENTITIES.write().get_mut_entity(id).unwrap().angular_velocity += ROTATION_SPEED;
    });
    input.bind_key(KeyCode::Space, InputTrigger::Down, |data : &mut InputData| {
        ENTITIES.write().get_mut_entity(data.target_id()).unwrap().stop();
    });

    // Editing
    input.bind_key(KeyCode::V, InputTrigger::Pressed, |data : &mut InputData| {
        let color = &mut data.edit_color;
        *color = (*color + 1) % MAX_COLOR;
    });
    input.bind_key(KeyCode::B, InputTrigger::Pressed, |data : &mut InputData| {
        let height = &mut data.edit_height;
        *height = (*height + 1) % MAX_HEIGHT;
    });
    input.bind_mouse(MouseButton::Left, InputTrigger::Down, |data : &mut InputData| {
        set_grid_cell(
            data.target_id,
            CAMERA.read().screen_to_world(mouse_pos()),
            ExternalPointer::new(Index(data.edit_color), data.edit_height)
        );
    });
    input.bind_key(KeyCode::F, InputTrigger::Pressed, |data : &mut InputData| {
        ENTITIES.write().get_mut_entity(data.target_id).unwrap().stop();
        data.target_id = (data.target_id + 1) % 2;
    });
    
    // Save/Load
    if !cfg!(target_arch = "wasm32") {
        input.bind_key(KeyCode::K, InputTrigger::Pressed, |data : &mut InputData| {
            let save_data = ENTITIES.read().save_entity(data.target_id);
            std::fs::write(&data.file_paths[data.target_id as usize], save_data).unwrap();
        });
        input.bind_key(KeyCode::L, InputTrigger::Pressed, |data : &mut InputData| {
            let mut entities = ENTITIES.write();
            let Ok(save_data) = std::fs::read_to_string(&data.file_paths[data.target_id as usize]) else {
                dbg!("No save data found");
                return;
            };
            *entities.get_mut_entity(data.target_id).unwrap() = Entity::load(save_data, data.target_id)
        });
    }

    // Debug
    input.bind_key(KeyCode::P, InputTrigger::Pressed, |_data : &mut InputData| {
        dbg!(GRAPH.read().nodes.internal_memory());
    });

    input
}
