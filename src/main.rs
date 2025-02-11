mod engine;
mod imports {
    use super::*;
    pub use engine::graph::{SparseDirectedGraph, GraphNode, BasicNode, ExternalPointer, Index};
    pub use engine::systems::io::{Camera,output::*};
    pub use engine::systems::collisions;
    pub use engine::systems::collisions::{Corners, corner_handling::*};
    pub use macroquad::math::{Vec2, UVec2, BVec2, IVec2, Mat2};
    pub use engine::utility::partition::{Aabb, grid::*};
    pub use super::{ID, Entity, Location};
    pub use macroquad::color::colors::*;
    pub use engine::utility::blocks::*;
    pub use macroquad::color::Color;
    pub use macroquad::input::*;
    pub use derive_new::new;
    pub use crate::GRAPH;
    pub use crate::CAMERA;
    pub use crate::ENTITIES;
    pub use crate::BLOCKS;
    pub use std::f32::consts::PI;
    pub use engine::utility::math::*;
    pub use engine::fancyintersection::*;
}
use imports::*;
use lazy_static::lazy_static;
use parking_lot::{RwLock, deadlock};
use std::time::Duration;
use std::thread;
lazy_static! {
    // Not sure how permanent these'll be, but they're here for now
    pub static ref GRAPH: RwLock<SparseDirectedGraph<BasicNode>> = RwLock::new(SparseDirectedGraph::new(4));
    pub static ref CAMERA: RwLock<Camera> = RwLock::new(Camera::new(
        Aabb::new(Vec2::ZERO, Vec2::splat(4.)), 
        0.9
    ));
    pub static ref ENTITIES: RwLock<EntityPool> = RwLock::new(EntityPool::new());
    // Temporary until I create a proper solution
    pub static ref BLOCKS: BlockPalette = BlockPalette::default();
}

// Constants
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
    pub pointer: ExternalPointer,
    pub position: Vec2,
}

pub type ID = u32;
// Chunk and store corner locations in u8s?
pub struct Entity {
    id : ID,
    location: Location,
    rotation: f32,
    forward: Vec2,
    velocity: Vec2,
    angular_velocity: f32,
    corners : Vec<Corners>,
}
impl Entity {
    pub fn new(id:ID, location:Location, orientation:f32) -> Self {
        Self {
            id,
            location,
            rotation: orientation,
            forward: Vec2::from_angle(orientation),
            velocity: Vec2::ZERO,
            angular_velocity: 0.0,
            corners: tree_corners(location.pointer),
        }
    }
    pub fn rel_rotate(&mut self, angle: f32) {
        self.rotation += angle;
        self.forward = Vec2::from_angle(self.rotation);
        self.recaclulate_corners();
    }
    pub fn set_rotation(&mut self, angle: f32) { 
        self.rotation = angle;
        self.forward = Vec2::from_angle(self.rotation);
        self.recaclulate_corners();
    }
    pub fn apply_forward_velocity(&mut self, speed:f32) { self.velocity += self.forward * speed }
    pub fn apply_perp_velocity(&mut self, speed:f32) { self.velocity += self.forward.perp() * speed }
    pub fn apply_abs_velocity(&mut self, delta:Vec2) { self.velocity += delta; }
    pub fn set_root(&mut self, new_root:ExternalPointer) { 
        self.location.pointer = new_root;
        self.recaclulate_corners();
    }
    pub fn recaclulate_corners(&mut self) { self.corners = tree_corners(self.location.pointer) }

}
#[derive(new)]
pub struct EntityPool {
    #[new(value = "Vec::new()")]
    entities: Vec<Entity>,
    #[new(value = "0")]
    id_counter: u32,
}
impl EntityPool {
    fn add_entity(&mut self, location:Location, orientation:f32) -> ID {
        self.id_counter += 1;
        self.entities.push(Entity::new(self.id_counter, location, orientation));
        self.id_counter
    }
    pub fn get_mut_entity(&mut self, id:ID) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|entity| entity.id == id)
    }
    pub fn get_entity(&self, id:ID) -> Option<&Entity> {
        self.entities.iter().find(|entity| entity.id == id)
    }
}

fn init_deadlock_detection() {
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1));
            let deadlocks = deadlock::check_deadlock();
            if !deadlocks.is_empty() {
                println!("{} deadlocks detected", deadlocks.len());
                for deadlock in deadlocks {
                    println!("Deadlock threads:");
                    for thread in deadlock {
                        println!("Thread Id {:#?}", thread.thread_id());
                        println!("Backtrace:\n{:#?}", thread.backtrace());
                    }
                }
                std::process::exit(1);
            }
        }
    });
}

#[macroquad::main("Window")]
async fn main() {
    init_deadlock_detection();
    #[cfg(debug_assertions)]
    println!("Debug mode");
    #[cfg(not(debug_assertions))]
    println!("Release mode");
    macroquad::window::request_new_screen_size(1024., 1024.);
    // Load world state once at startup
    dbg!((1.17549435E-38).snap_zero());
    let world_pointer = {
        let string = std::fs::read_to_string("src/save.json").unwrap_or_default();
        if string.is_empty() { 
            GRAPH.write().get_root(0, 3)
        } else { 
            GRAPH.write().load_object_json(string)
        }
    };
    
    let terrain = ENTITIES.write().add_entity(
        Location::new(world_pointer, TERRAIN_SPAWN),
        TERRAIN_ROTATION_SPAWN,
    );

    let player = {
        //Graph has to be unlocked before add_entity is called so corners can be read
        let root = GRAPH.write().get_root(3, 0);
        ENTITIES.write().add_entity(
            Location::new(root, PLAYER_SPAWN),
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
            // if is_key_down(KeyCode::Q) { player_entity.rel_rotate(-PLAYER_ROTATION_SPEED); }
            // if is_key_down(KeyCode::E) { player_entity.rel_rotate(PLAYER_ROTATION_SPEED); }
            if is_key_down(KeyCode::Q) { player_entity.angular_velocity -= PLAYER_ROTATION_SPEED; }
            if is_key_down(KeyCode::E) { player_entity.angular_velocity += PLAYER_ROTATION_SPEED; }
            if is_key_down(KeyCode::Space) { 
                player_entity.velocity = Vec2::ZERO;
                player_entity.angular_velocity = 0.0;
            }
            if is_mouse_button_down(MouseButton::Left) {
                let mouse_pos = CAMERA.read().screen_to_world(Vec2::from(mouse_position()));
                
                let terrain_entity = entities.get_mut_entity(terrain).unwrap();
                let mouse_pos = (mouse_pos - terrain_entity.location.position).rotate(Vec2::from_angle(-terrain_entity.rotation)) + terrain_entity.location.position;
                
                if let Some(pointer) = set_grid_cell(
                    ExternalPointer::new(Index(color), height),
                    mouse_pos,
                    terrain_entity.location
                ) { terrain_entity.set_root(pointer) }
            }
        }

        if is_key_pressed(KeyCode::V) { color = (color + 1) % MAX_COLOR; }
        if is_key_pressed(KeyCode::B) { height = (height + 1) % MAX_HEIGHT; }
        
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
                let old_removal = engine::graph::bfs_nodes(
                    graph.nodes.internal_memory(),
                    terrain_entity.location.pointer.pointer,
                    3
                );
                graph.mass_remove(&old_removal);
                new_pointer
            };
            terrain_entity.set_root(new_pointer);
        }
        
        render::draw_all(true);
        
        // We want to move the camera to where the player is drawn, not where the player is moved to.
        let player_pos = ENTITIES.read().get_entity(player).unwrap().location.position;
        
        collisions::n_body_collisions(terrain).await;
        
        // We don't want to move the camera until after we've drawn all the collision debug
        CAMERA.write().update(player_pos, 1.);
        
        macroquad::window::next_frame().await
    }

}

pub fn set_grid_cell(to:ExternalPointer, world_point:Vec2, location:Location) -> Option<ExternalPointer> {
    if to.height > location.pointer.height { return None; }
    let cell = gate::point_to_cells(location, to.height, world_point)[0]?; 
    let path = ZorderPath::from_cell(cell, location.pointer.height - to.height);
    GRAPH.write().set_node(location.pointer, &path.steps(), to.pointer).ok()
}
