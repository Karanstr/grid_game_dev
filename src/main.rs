mod engine;
mod globals {
    use crate::engine::blocks::BlockPalette;
    use crate::engine::grid::dag::{SparseDirectedGraph, BasicNode};
    use crate::engine::camera::Camera;
    use macroquad::math::Vec2;
    use crate::engine::entities::EntityPool;
    use lazy_static::lazy_static;
    use parking_lot::RwLock;
    lazy_static! {
        pub static ref GRAPH: RwLock<SparseDirectedGraph<BasicNode>> = RwLock::new(SparseDirectedGraph::<BasicNode>::new(4));
        pub static ref ENTITIES: RwLock<EntityPool> = RwLock::new(EntityPool::new());
        pub static ref CAMERA: RwLock<Camera> = RwLock::new(Camera::new(Vec2::ZERO, 4.));
        pub static ref BLOCKS: BlockPalette = BlockPalette::default();
    }
}
use globals::*;
use engine::input::*;
use macroquad::math::Vec2;
use macroquad::prelude::{mouse_position, KeyCode, MouseButton};
use std::f32::consts::PI;
use engine::{
    physics::collisions::n_body_collisions,
    entities::{Entity, ID, Location},
    math::Aabb,
    grid::dag::{Index, ExternalPointer},
    grid::partition::{gate, ZorderPath},
};

use std::time::Duration;
use std::thread;
fn init_deadlock_detection() {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(3));
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

const SPEED: f32 = 0.005;
const ROTATION_SPEED: f32 = PI/512.;
const MAX_COLOR: usize = 4;
const MAX_HEIGHT: u32 = 4;

fn set_panic_hook() {
    std::panic::set_hook(Box::new(|panic_info| {
        // Get the panic message
        let message = panic_info.payload().downcast_ref::<String>()
            .map(|s| s.as_str())
            .or_else(|| panic_info.payload().downcast_ref::<&str>().copied())
            .unwrap_or("<no message>");

        // Get the location
        let location = panic_info.location()
            .map(|loc| format!("{}:{}:{}", loc.file(), loc.line(), loc.column()))
            .unwrap_or_else(|| "<unknown location>".to_string());

        eprintln!("\nPanic occurred:\n");
        eprintln!("Message: {}", message);
        eprintln!("Location: {}", location);
        eprintln!("\nBacktrace:\n{}", std::backtrace::Backtrace::capture());
        
        // Exit with error code 1
        std::process::exit(1);
    }));
}

fn mouse_pos() -> Vec2 { Vec2::from(mouse_position()) }
use macroquad::color::*;

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
    // Load entities 
    {
        let mut entity_pool = ENTITIES.write();
        let terrain_string = if cfg!(target_arch = "wasm32") { 
            String::from_utf8(include_bytes!("../data/terrain.json").as_ref().to_vec()).unwrap_or_default()
        } else {
            std::fs::read_to_string("data/terrain.json").unwrap_or_default()
        };
        entity_pool.add_to_pool(
            Entity::load(terrain_string, 0)
        );
        let player_string = if cfg!(target_arch = "wasm32") { 
            String::from_utf8(include_bytes!("../data/player.json").as_ref().to_vec()).unwrap_or_default()
        } else {
            std::fs::read_to_string("data/player.json").unwrap_or_default()
        };
        entity_pool.add_to_pool(
            Entity::load(player_string, 1)
        );
    }
    
    let mut vars = InputData::default();
    let mut input = set_key_binds();
    
    loop {
        let old_pos = { // Drop entities after reading from it
            let entities = ENTITIES.read();
            entities.draw_all(vars.render_rotated, vars.render_debug);
            let target = entities.get_entity(vars.target_id()).unwrap();
            target.draw_outline(macroquad::color::DARKBLUE);
            // let location = entities.get_entity((vars.target_id() + 1) % 2).unwrap().location;
            // if let Some(aabb) = target.aabb() { 
            //     aabb.overlaps(location);
            //     CAMERA.read().outline_bounds(aabb, 0.3, macroquad::color::DARKBLUE);
            // }
            // We want to move the camera to where the target is drawn, not where the target is moved to.
            target.location.position
        };
        
        
        input.handle(&mut vars);
        
        n_body_collisions((vars.target_id() + 1) % 2);
        
        // We don't want to move the camera until after we've drawn all the collision debug.
        // This ensures everything lines up with the current frame.
        // CAMERA.write().update(Some((old_pos, 0.4)));
        CAMERA.write().update(None);
        macroquad::window::next_frame().await
    }

}

impl Aabb {
    pub fn overlaps(&self, location:Location) {
        let top_left = self.min();
        let bottom_right = self.max();
        let corners = [
            top_left,
            Vec2::new(bottom_right.x, top_left.y),
            bottom_right,
            Vec2::new(top_left.x, bottom_right.y),
        ];
        let cells = corners.iter()
            .filter_map(|corner| gate::point_to_real_cells(location, *corner)[0]);
        let points = cells.map(|cell| {
            cell.to_point(location, Vec2::ONE)
        });
        for point in points {
            CAMERA.read().draw_point(point, 0.2, Color::from_rgba(255, 0, 0, 150));
        }
    }
}

pub fn set_grid_cell(entity:ID, world_point:Vec2, new_cell:ExternalPointer) {
    let mut entities = ENTITIES.write();
    let entity = &mut entities.get_mut_entity(entity).unwrap();
    if new_cell.height > entity.location.pointer.height { return; }
    
    let rotated_point = (world_point - entity.location.position).rotate(Vec2::from_angle(-entity.rotation)) + entity.location.position;
    
    let Some(cell) = gate::point_to_cells(entity.location, new_cell.height, rotated_point)[0] else { return };
    let path = ZorderPath::from_cell(cell, entity.location.pointer.height - new_cell.height);
    let Ok(root) = GRAPH.write().set_node(entity.location.pointer, &path.steps(), new_cell.pointer) else {
        dbg!("Failed to set cell");
        return;
    };
    entity.set_root(root);
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
    pub render_debug : bool,
    pub render_rotated: bool,
    pub file_paths : [String; 2],
}
impl Default for InputData {
    fn default() -> Self {
        Self {
            target_id: 1,
            edit_color: 0,
            edit_height: 0,
            render_debug: true,
            render_rotated: true,
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
    input.bind_key(KeyCode::O, InputTrigger::Pressed, |data : &mut InputData| {
        data.render_debug = !data.render_debug;
    });
    input.bind_key(KeyCode::I, InputTrigger::Pressed, |data : &mut InputData| {
        data.render_rotated = !data.render_rotated;
    });

    // Camera Controls
    input.bind_key(KeyCode::Equal, InputTrigger::Down, |_data : &mut InputData| {
        CAMERA.write().change_zoom(1.02);
    });
    input.bind_key(KeyCode::Minus, InputTrigger::Down, |_data : &mut InputData| {
        CAMERA.write().change_zoom(1./1.02);
    });

    input
}
