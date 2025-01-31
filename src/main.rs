mod engine;

// Game constants
const PLAYER_SPEED: f32 = 0.01;
const PLAYER_ROTATION_SPAWN: f32 = 0.;
const TERRAIN_ROTATION_SPAWN: f32 = 0.;
const MAX_COLOR: usize = 4;
const MAX_HEIGHT: u32 = 4;
const WINDOW_SIZE: f32 = 512.0;

mod imports {
    use super::*;
    pub use engine::graph::{SparseDirectedGraph, GraphNode, BasicNode, ExternalPointer, Index};
    pub use engine::systems::io::{Camera,output::*};
    pub use engine::systems::collisions;
    pub use engine::systems::collisions::{Corners, corner_handling::*};
    pub use macroquad::math::{Vec2, UVec2, BVec2, IVec2, Mat2};
    pub use engine::utility::partition::{AABB, grid::*};
    pub use super::{ID, Entity, Location};
    pub use macroquad::color::colors::*;
    pub use engine::utility::blocks::*;
    pub use macroquad::color::Color;
    pub use macroquad::input::*;
    pub use derive_new::new;
    pub use crate::GRAPH;
    pub use crate::CAMERA;
    pub use crate::ENTITIES;
    pub use std::f32::consts::PI;
}
use imports::*;
use lazy_static::lazy_static;
use parking_lot::{RwLock, deadlock};
use std::time::Duration;
use std::thread;
lazy_static! {
    pub static ref GRAPH: RwLock<SparseDirectedGraph<BasicNode>> = RwLock::new(SparseDirectedGraph::new(4));
    pub static ref CAMERA: RwLock<Camera> = RwLock::new(Camera::new(
        AABB::new(Vec2::ZERO, Vec2::splat(4.)), 
        0.9
    ));
    pub static ref ENTITIES: RwLock<EntityPool> = RwLock::new(EntityPool::new());
}

//Add a method which updates the location of an entity and handles corner recalculation

#[derive(Debug, Clone, Copy, new)]
pub struct Location {
    pub pointer: ExternalPointer,
    pub position: Vec2,
}

//Chunk and store corner locations in u8s?
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ID(pub u32);
pub struct Entity {
    id : ID,
    location: Location,
    forward: Vec2,
    rotation: f32,
    velocity: Vec2,
    // angular_velocity: f32,
    corners : Vec<Corners>,
}
impl Entity {
    pub fn new(id:ID, location:Location, orientation:f32) -> Self {
        Self {
            id,
            location,
            forward: Vec2::from_angle(orientation),
            rotation: orientation,
            velocity: Vec2::ZERO,
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
        self.entities.push(Entity::new(ID(self.id_counter), location, orientation));
        ID(self.id_counter)
    }
    pub fn get_mut_entity(&mut self, id:ID) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|entity| entity.id == id)
    }
    pub fn get_entity(&self, id:ID) -> Option<&Entity> {
        self.entities.iter().find(|entity| entity.id == id)
    }
}

// Initialize deadlock detection on program start
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
    macroquad::window::request_new_screen_size(WINDOW_SIZE, WINDOW_SIZE);
    let blocks = BlockPalette::new();
    // Load world state once at startup
    let world_pointer = {
        let string = std::fs::read_to_string("src/save.json").unwrap_or_default();
        let mut graph = GRAPH.write();
        if string.is_empty() { 
            graph.get_root(0, 3)
        } else { 
            graph.load_object_json(string)
        }
    };
    
    let terrain = ENTITIES.write().add_entity(
        Location::new(world_pointer, Vec2::new(0., 0.)),
        TERRAIN_ROTATION_SPAWN,
    );

    let player = {
        //Graph has to be unlocked before add_entity is called so entity corners can be read from the graph
        let root = GRAPH.write().get_root(3, 0);
        ENTITIES.write().add_entity(
            Location::new(
                root,
                Vec2::new(0., 0.)
            ),
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
            if is_key_down(KeyCode::Space) { player_entity.velocity = Vec2::ZERO }
            
            if is_mouse_button_down(MouseButton::Left) {
                let mouse_pos = CAMERA.read().screen_to_world(Vec2::from(mouse_position()));
                
                let terrain_entity = entities.get_mut_entity(terrain).unwrap();
                let mouse_pos = (mouse_pos - terrain_entity.location.position).rotate(Vec2::from_angle(-terrain_entity.rotation)) + terrain_entity.location.position;
                let new_pointer = GRAPH.write().get_root(color, height);
                
                if let Some(pointer) = set_grid_cell(new_pointer, mouse_pos, terrain_entity.location) {
                    terrain_entity.set_root(pointer);
                }
            }
        }

        if is_key_pressed(KeyCode::V) { color = (color + 1) % MAX_COLOR; }
        if is_key_pressed(KeyCode::B) { height = (height + 1) % MAX_HEIGHT; }
        
        if is_key_pressed(KeyCode::P) { 
            dbg!(GRAPH.read().nodes.internal_memory()); 
        }
        
        if is_key_pressed(KeyCode::K) {
            let entities = ENTITIES.write();
            let save_data = {
                let terrain_entity = entities.get_entity(terrain).unwrap();
                GRAPH.read().save_object_json(terrain_entity.location.pointer)
            };
            std::fs::write("src/save.json", save_data).unwrap();
        }
        
        if is_key_pressed(KeyCode::L) {
            let mut entities = ENTITIES.write();
            let terrain_entity = entities.get_mut_entity(terrain).unwrap();
            let mut graph = GRAPH.write();
            
            let save_data = std::fs::read_to_string("src/save.json").unwrap();
            let new_pointer = graph.load_object_json(save_data);
            
            let old_removal = engine::graph::bfs_nodes(
                &graph.nodes.internal_memory(),
                terrain_entity.location.pointer.pointer,
                3
            );
            graph.mass_remove(&old_removal);
            terrain_entity.set_root(new_pointer);
        }
        
        render::draw_all(&blocks, true);
        
        let player_pos = ENTITIES.read().get_entity(player).unwrap().location.position;
        
        collisions::n_body_collisions(terrain);

        CAMERA.write().update(player_pos, 0.1);

        macroquad::window::next_frame().await
    }

}

pub fn set_grid_cell(to:ExternalPointer, world_point:Vec2, location:Location) -> Option<ExternalPointer> {
    let height = to.height;
    if height <= location.pointer.height {
        let Some(cell) = gate::point_to_cells(location, height, world_point)[0] else { return None };
        let path = ZorderPath::from_cell(cell, location.pointer.height - height);
        if let Ok(pointer) = GRAPH.write().set_node(location.pointer, &path.steps(), to.pointer) {
            return Some(pointer);
        }
    }
    None
}
