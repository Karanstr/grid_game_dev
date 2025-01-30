mod engine;
mod imports {
    use super::*;
    pub use engine::graph::{SparseDirectedGraph, GraphNode, BasicNode, ExternalPointer, Index};
    pub use engine::systems::io::{Camera,output::*};
    pub use engine::systems::collisions;
    pub use engine::systems::collisions::{Corners, corner_handling::*};
    pub use macroquad::math::{Vec2, UVec2, BVec2, IVec2, Mat2};
    pub use engine::utility::partition::{AABB, grid::*};
    pub use super::{EntityPool, ID, Entity, Location};
    pub use macroquad::color::colors::*;
    pub use engine::utility::blocks::*;
    pub use macroquad::color::Color;
    pub use macroquad::input::*;
    pub use derive_new::new;
    pub use crate::GRAPH;
    pub use crate::CAMERA;
    pub use std::f32::consts::PI;
}
use imports::*;
use lazy_static::lazy_static;
use std::sync::RwLock;
lazy_static! {
    pub static ref GRAPH: RwLock<SparseDirectedGraph<BasicNode>> = RwLock::new(SparseDirectedGraph::new(4));
    pub static ref CAMERA: RwLock<Camera> = RwLock::new(Camera::new(
        AABB::new(Vec2::ZERO, Vec2::splat(4.)), 
        0.9
    ));
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
        self.corners = tree_corners(self.location.pointer);
    }
    pub fn set_rotation(&mut self, angle: f32) { 
        self.rotation = angle;
        self.forward = Vec2::from_angle(self.rotation);
        self.corners = tree_corners(self.location.pointer);
    }
    pub fn apply_forward_velocity(&mut self, speed:f32) { self.velocity += self.forward * speed }
    pub fn apply_perp_velocity(&mut self, speed:f32) { self.velocity += self.forward.perp() * speed }
    pub fn apply_abs_velocity(&mut self, delta:Vec2) { self.velocity += delta; }
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
    fn get_mut_entity(&mut self, id:ID) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|entity| entity.id == id)
    }
    fn get_entity(&self, id:ID) -> Option<&Entity> {
        self.entities.iter().find(|entity| entity.id == id)
    }
}

#[macroquad::main("Window")]
async fn main() {
    macroquad::window::request_new_screen_size(512., 512.);
    let mut entities = EntityPool::new();
    let blocks = BlockPalette::new();
    // Load world state once at startup
    let world_pointer = {
        let string = std::fs::read_to_string("src/save.json").unwrap_or_default();
        let mut graph = GRAPH.write().unwrap();
        if string.is_empty() { 
            graph.get_root(0, 3)
        } else { 
            graph.load_object_json(string)
        }
    };
    
    let rotation = 0.8;
    let terrain = entities.add_entity(
        Location::new(world_pointer, Vec2::new(0., 0.)),
        rotation,
    );
    
    // Initialize player
    let player = {
        let new_root = GRAPH.write().unwrap().get_root(3, 0);
        entities.add_entity(
            Location::new(
                new_root,
                Vec2::new(0., 0.)
            ),
            rotation + 0.3 + PI,
        )
    };
    
    let mut color = 0;
    let mut height = 0;

    loop {
        let player_entity = entities.get_mut_entity(player).unwrap();
        let speed = 0.01;
        // let rot_speed = 0.1;
        // if is_key_down(KeyCode::A) { player_entity.rel_rotate(-rot_speed); }
        // if is_key_down(KeyCode::D) { player_entity.rel_rotate(rot_speed); }
        // if is_key_down(KeyCode::W) { player_entity.move_forward(speed); }
        // if is_key_down(KeyCode::S) { player_entity.move_forward(-speed); }
        // player_entity.apply_abs_velocity(Vec2::new(0., speed/2.));
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) { player_entity.apply_perp_velocity(-speed); }
        if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) { player_entity.apply_perp_velocity(speed); }
        if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) { player_entity.apply_forward_velocity(-speed); }
        if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) { player_entity.apply_forward_velocity(speed); }
        if is_key_down(KeyCode::Space) { player_entity.velocity = Vec2::ZERO }
        
        // Handle mouse input with minimized lock scope
        if is_mouse_button_down(MouseButton::Left) {
            let mouse_pos = {
                let camera = CAMERA.read().unwrap();
                camera.screen_to_world(Vec2::from(mouse_position()))
            };
            
            let terrain_entity = entities.get_mut_entity(terrain).unwrap();
            let new_pointer = { GRAPH.write().unwrap().get_root(color, height) };
            
            if let Some(pointer) = set_grid_cell(new_pointer, mouse_pos, terrain_entity.location) {
                terrain_entity.location.pointer = pointer;
                terrain_entity.corners = tree_corners(pointer);
            }
        }

        if is_key_pressed(KeyCode::V) { color += 1; color %= 4; }
        if is_key_pressed(KeyCode::B) { height += 1; height %= 4; }
        
        // Debug output with minimized lock scope
        if is_key_pressed(KeyCode::P) { 
            let graph = GRAPH.read().unwrap();
            dbg!(graph.nodes.internal_memory()); 
        }
        
        // Save game state with minimized lock scope
        if is_key_pressed(KeyCode::K) {
            let save_data = {
                let graph = GRAPH.read().unwrap();
                let terrain_entity = entities.get_entity(terrain).unwrap();
                graph.save_object_json(terrain_entity.location.pointer)
            };
            std::fs::write("src/save.json", save_data).unwrap();
        }
        
        // Load game state with minimized lock scope
        if is_key_pressed(KeyCode::L) {
            let terrain_entity = entities.get_mut_entity(terrain).unwrap();
            let mut graph = GRAPH.write().unwrap();
            
            let save_data = std::fs::read_to_string("src/save.json").unwrap();
            let new_pointer = graph.load_object_json(save_data);
            
            let old_removal = engine::graph::bfs_nodes(
                &graph.nodes.internal_memory(),
                terrain_entity.location.pointer.pointer,
                3
            );
            terrain_entity.location.pointer = new_pointer;
            graph.mass_remove(&old_removal);
            terrain_entity.corners = tree_corners(new_pointer);
        }
        
        //Move before rendering
        
        // Rendering with minimized lock scope
        render::draw_all(&entities, &blocks, true);
        let (player_pos, player_forward) = {
            let player_entity = entities.get_entity(player).unwrap();
            (player_entity.location.position, player_entity.forward)
        };

        collisions::n_body_collisions(&mut entities, terrain);

        {
            let mut camera = CAMERA.write().unwrap();
            camera.draw_vec_line(
                player_pos,
                player_pos + player_forward / 2.,
                0.05, WHITE
            );
            //Move camera after rendering everything
            camera.update(player_pos, 0.1);
        }

        macroquad::window::next_frame().await
    }
}

pub fn set_grid_cell(to:ExternalPointer, world_point:Vec2, location:Location) -> Option<ExternalPointer> {
    let height = to.height;
    if height <= location.pointer.height {
        let cell = gate::point_to_cells(location, height, world_point)[0];
        if let Some(cell) = cell {
            let path = ZorderPath::from_cell(cell, location.pointer.height - height);
            if let Ok(pointer) = GRAPH.write().unwrap().set_node(location.pointer, &path.steps(), to.pointer) {
                return Some(pointer);
            }
        }
    }
    None
}
