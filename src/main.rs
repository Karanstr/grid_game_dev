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
    use parking_lot::RwLock;
    use lazy_static::lazy_static;
    use crate::engine::blocks::BlockPalette;
    use crate::engine::grid::dag::{SparseDirectedGraph, BasicNode};
    use crate::engine::camera::Camera;
    use macroquad::math::Vec2;
    use crate::engine::math::Aabb;
    use crate::engine::entities::EntityPool;
    lazy_static! { 
        pub static ref BLOCKS: BlockPalette = BlockPalette::default();
        pub static ref CAMERA: RwLock<Camera> = RwLock::new(Camera::new(
            Aabb::new(Vec2::ZERO, Vec2::splat(8.)), 
            0.9
        ));
        pub static ref ENTITIES: RwLock<EntityPool> = RwLock::new(EntityPool::new());
        pub static ref GRAPH: RwLock<SparseDirectedGraph<BasicNode>> = RwLock::new(SparseDirectedGraph::<BasicNode>::new(4));
    }
}
use globals::*;

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

fn mouse_pos() -> Vec2 { Vec2::from(mouse_position()) }

#[macroquad::main("Window")]
async fn main() {
    init_deadlock_detection();
    set_panic_hook();
    #[cfg(debug_assertions)]
    println!("Debug mode");
    #[cfg(not(debug_assertions))]
    println!("Release mode");
    macroquad::window::request_new_screen_size(1024., 1024.);
    
    // Load world state once at startup
    let world_pointer = {
        let string = std::fs::read_to_string("src/save.json").unwrap_or_default();
        if string.is_empty() { 
            GRAPH.write().get_root(0, 3)
        } else { 
            GRAPH.write().load_object_json(string)
        }
    };
    
    let terrain = ENTITIES.write().add_entity(
        Location::new(TERRAIN_SPAWN, world_pointer),
        TERRAIN_ROTATION_SPAWN,
    );

    let player = {
        //Graph has to be unlocked before add_entity is called so corners can be read
        let root = GRAPH.write().get_root(3, 0);
        ENTITIES.write().add_entity(
            Location::new(PLAYER_SPAWN, root),
            PLAYER_ROTATION_SPAWN,
        )
    };
    
    let mut color = 0;
    let mut height = 0;

    loop {
        { // Scoped to visualize the lock of entities
            let mut entities = ENTITIES.write();
            let player_entity = entities.get_mut_entity(player).unwrap();
            if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) { player_entity.apply_abs_velocity(Vec2::new(0., -PLAYER_SPEED)); }
            if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) { player_entity.apply_abs_velocity(Vec2::new(0., PLAYER_SPEED)); }
            if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) { player_entity.apply_abs_velocity(Vec2::new(-PLAYER_SPEED, 0.)); }
            if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { player_entity.apply_abs_velocity(Vec2::new(PLAYER_SPEED, 0.)); }
            if is_key_down(KeyCode::Q) { player_entity.angular_velocity -= PLAYER_ROTATION_SPEED; }
            if is_key_down(KeyCode::E) { player_entity.angular_velocity += PLAYER_ROTATION_SPEED; }
            if is_key_down(KeyCode::Space) { 
                player_entity.velocity = Vec2::ZERO;
                player_entity.angular_velocity = 0.0;
            }
        }
        if is_key_pressed(KeyCode::V) { color = (color + 1) % MAX_COLOR; }
        if is_key_pressed(KeyCode::B) { height = (height + 1) % MAX_HEIGHT; }
        if is_mouse_button_down(MouseButton::Left) {
            set_grid_cell(
                terrain,
                CAMERA.read().screen_to_world(mouse_pos()),
                ExternalPointer::new(Index(color), height)
            );
        }
        
        if is_key_pressed(KeyCode::P) { dbg!(GRAPH.read().nodes.internal_memory()); }
        
        if is_key_pressed(KeyCode::K) {
            let save_data = GRAPH.read().save_object_json(ENTITIES.read().get_entity(terrain).unwrap().location.pointer);
            std::fs::write("src/save.json", save_data).unwrap();
        }
        
        if is_key_pressed(KeyCode::L) {
            let mut entities = ENTITIES.write();
            let terrain_entity = entities.get_mut_entity(terrain).unwrap();
            let new_pointer = {
                let mut graph = GRAPH.write();
                let save_data = std::fs::read_to_string("src/save.json").unwrap();
                let new_pointer = graph.load_object_json(save_data);
                let old_removal = engine::grid::dag::bfs_nodes(
                    graph.nodes.internal_memory(),
                    terrain_entity.location.pointer.pointer,
                    3
                );
                graph.mass_remove(&old_removal);
                new_pointer
            };
            terrain_entity.set_root(new_pointer);
        }
        
        ENTITIES.read().draw_all(true);
        
        // We want to move the camera to where the player is drawn, not where the player is moved to.
        let player_pos = ENTITIES.read().get_entity(player).unwrap().location.position;
        
        // Move n_body_collisions into entitypool? 
        n_body_collisions(terrain).await;
        
        // We don't want to move the camera until after we've drawn all the collision debug.
        // This ensures everything lines up with the current frame.
        CAMERA.write().update(player_pos, 1.);
        
        macroquad::window::next_frame().await
    }

}


pub fn set_grid_cell(entity:ID, world_point:Vec2, new_cell:ExternalPointer) {
    let mut entities = ENTITIES.write();
    let location = &mut entities.get_mut_entity(entity).unwrap().location;
    if new_cell.height > location.pointer.height { return; }
    let Some(cell) = gate::point_to_cells(*location, new_cell.height, world_point)[0] else { return };
    let path = ZorderPath::from_cell(cell, location.pointer.height - new_cell.height);
    location.pointer = GRAPH.write().set_node(location.pointer, &path.steps(), new_cell.pointer).unwrap();
}
    