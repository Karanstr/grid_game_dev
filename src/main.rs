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
    use static_init::dynamic;
    #[dynamic(drop)]
    pub static mut GRAPH: SparseDirectedGraph<BasicNode> = SparseDirectedGraph::<BasicNode>::new(4);
    #[dynamic(drop)]
    pub static mut ENTITIES: EntityPool = EntityPool::new();
    // Must be lazy so camera can access miniquad screen resources
    #[dynamic(drop, lazy)]
    pub static mut CAMERA: Camera = Camera::new(
        Aabb::new(Vec2::ZERO, Vec2::splat(8.)), 
        0.9
    );
    #[dynamic()]
    pub static BLOCKS: BlockPalette = BlockPalette::default();
}
use globals::*;
// use parking_lot::RwLock;
// use std::sync::Arc;
use engine::input::*;
use static_init::dynamic;
#[dynamic]
pub static mut EDIT_COLOR: usize = 0;
#[dynamic]
pub static mut EDIT_HEIGHT: u32 = 0;
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
    #[cfg(debug_assertions)]
    println!("Debug mode");
    #[cfg(not(debug_assertions))]
    println!("Release mode");
    macroquad::window::request_new_screen_size(1024., 1024.);
    
    // Load world state once at startup
    let world_pointer = {
        let string = std::fs::read_to_string("data/save.json").unwrap_or_default();
        if string.is_empty() { 
            GRAPH.write().get_root(0, 3)
        } else { 
            GRAPH.write().load_object_json(string)
        }
    };
    
    let terrain_id = ENTITIES.write().add_entity(
        Location::new(TERRAIN_SPAWN, world_pointer),
        TERRAIN_ROTATION_SPAWN,
    );

    let player_id = {
        //Graph has to be unlocked before add_entity is called so corners can be read from it
        let player_pointer = GRAPH.write().get_root(3, 0);
        ENTITIES.write().add_entity(
            Location::new(PLAYER_SPAWN, player_pointer),
            PLAYER_ROTATION_SPAWN,
        )
    };

    
    // let color = Arc::new(0);
    // let height = Arc::new(0);
    
    let mut input = InputHandler::new();
    { // Input bindings
        // Movement
        input.bind_key(KeyCode::W, InputTrigger::Down, move || {
            ENTITIES.write().get_mut_entity(player_id).unwrap().apply_abs_velocity(Vec2::new(0., -PLAYER_SPEED));
        });
        input.bind_key(KeyCode::S, InputTrigger::Down, move || {
            ENTITIES.write().get_mut_entity(player_id).unwrap().apply_abs_velocity(Vec2::new(0., PLAYER_SPEED));
        });
        input.bind_key(KeyCode::A, InputTrigger::Down, move || {
            ENTITIES.write().get_mut_entity(player_id).unwrap().apply_abs_velocity(Vec2::new(-PLAYER_SPEED, 0.));
        });
        input.bind_key(KeyCode::D, InputTrigger::Down, move || {
            ENTITIES.write().get_mut_entity(player_id).unwrap().apply_abs_velocity(Vec2::new(PLAYER_SPEED, 0.));
        });
        input.bind_key(KeyCode::Q, InputTrigger::Down, move || {
            ENTITIES.write().get_mut_entity(player_id).unwrap().angular_velocity -= PLAYER_ROTATION_SPEED;
        });
        input.bind_key(KeyCode::E, InputTrigger::Down, move || {
            ENTITIES.write().get_mut_entity(player_id).unwrap().angular_velocity += PLAYER_ROTATION_SPEED;
        });
        input.bind_key(KeyCode::Space, InputTrigger::Down, move || {
            let mut entities = ENTITIES.write();
            entities.get_mut_entity(player_id).unwrap().velocity = Vec2::ZERO;
            entities.get_mut_entity(player_id).unwrap().angular_velocity = 0.0;
        });

        // Editing
        input.bind_key(KeyCode::V, InputTrigger::Pressed, move || {
            let mut color = EDIT_COLOR.write();
            *color = (*color + 1) % MAX_COLOR;
        });
        input.bind_key(KeyCode::B, InputTrigger::Pressed, move || {
            let mut height = EDIT_HEIGHT.write();
            *height = (*height + 1) % MAX_HEIGHT;
        });
        input.bind_mouse(MouseButton::Left, InputTrigger::Down, move || {
            set_grid_cell(
                terrain_id,
                CAMERA.read().screen_to_world(mouse_pos()),
                ExternalPointer::new(Index(*EDIT_COLOR.read()), *EDIT_HEIGHT.read())
            );
        });
        
        // Save/Load
        input.bind_key(KeyCode::K, InputTrigger::Pressed, move || {
            let save_data = GRAPH.read().save_object_json(ENTITIES.read().get_entity(terrain_id).unwrap().location.pointer);
            std::fs::write("data/save.json", save_data).unwrap();
        });
        input.bind_key(KeyCode::L, InputTrigger::Pressed, move || {
            let mut entities = ENTITIES.write();
            let terrain_entity = entities.get_mut_entity(terrain_id).unwrap();
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

    }
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

// pub fn set_key_binds() -> InputHandler {
    
// }


pub fn set_grid_cell(entity:ID, world_point:Vec2, new_cell:ExternalPointer) {
    let mut entities = ENTITIES.write();
    let entity = &mut entities.get_mut_entity(entity).unwrap();
    if new_cell.height > entity.location.pointer.height { return; }
    let Some(cell) = gate::point_to_cells(entity.location, new_cell.height, world_point)[0] else { return };
    let path = ZorderPath::from_cell(cell, entity.location.pointer.height - new_cell.height);
    if let Ok(root) = GRAPH.write().set_node(entity.location.pointer, &path.steps(), new_cell.pointer) {
        entity.set_root(root);
    } else { dbg!("Failed to set cell"); }
}