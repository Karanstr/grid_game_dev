mod engine;
mod imports {
    use super::*;
    pub use macroquad::math::{Vec2, UVec2, BVec2, IVec2};
    pub use macroquad::miniquad::window::screen_size;
    pub use derive_new::new;
    pub use engine::graph::{SparseDirectedGraph, GraphNode, ExternalPointer, Index};
    pub use engine::utility::partition::AABB;
    pub use engine::utility::partition::grid::*;
    pub use engine::systems::collisions;
    pub use macroquad::color::colors::*;
    pub use macroquad::color::Color;
    pub use macroquad::shapes::*;
    pub use engine::systems::io::Camera;
    pub use super::{EntityPool, ID, Entity, Location};
    pub use engine::utility::blocks::*;
    pub use macroquad::input::*;
    pub use output::*;
    pub use engine::systems::io::*;
    // pub use engine::systems::collisions::*;

}
use imports::*;

#[derive(Debug, Clone, Copy, new)]
pub struct Location {
    pub pointer: ExternalPointer,
    pub position: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ID(pub u32);
#[derive(new)]
pub struct Entity {
    id : ID,
    location: Location,
    velocity: Vec2,
}
#[derive(new)]
pub struct EntityPool {
    #[new(value = "Vec::new()")]
    entities: Vec<Entity>,
    #[new(value = "0")]
    id_counter: u32,
}
impl EntityPool {
    fn add_entity(&mut self, location:Location, velocity:Vec2) -> ID {
        self.id_counter += 1;
        self.entities.push(Entity::new(ID(self.id_counter), location, velocity));
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
    let mut camera = Camera::new( AABB::new(Vec2::ZERO, Vec2::splat(4.)), 0.9);
    let mut graph = SparseDirectedGraph::<engine::graph::BasicNode>::new(4);
    let blocks = BlockPalette::new();
    let terrain = entities.add_entity(
        Location::new(
            graph.get_root(0, 3),
            Vec2::new(0., 0.)
        ), Vec2::ZERO
    );
    let player = entities.add_entity(
        Location::new(
            graph.get_root(3, 0),
            Vec2::new(-2., 0.)
        ),
        Vec2::ZERO,
    );
    let mut color = 0;
    let mut height = 0;
    loop {
        // Change this to an input module
        let player_entity = entities.get_mut_entity(player).unwrap();
        let speed = 0.01;
        if is_key_down(KeyCode::W) { player_entity.velocity.y -= speed; }
        if is_key_down(KeyCode::S) { player_entity.velocity.y += speed; }
        if is_key_down(KeyCode::A) { player_entity.velocity.x -= speed; }
        if is_key_down(KeyCode::D) { player_entity.velocity.x += speed; }
        if is_key_pressed(KeyCode::V) { color += 1; color %= 4;}
        if is_key_pressed(KeyCode::B) { height += 1; height %= 4; }
        if is_key_pressed(KeyCode::P) { dbg!(graph.nodes.internal_memory()); }
        input::handle_mouse_input(&camera, &mut graph, &mut entities.get_mut_entity(terrain).unwrap().location, color, height);
        render::draw_all(&camera, &graph, &entities, &blocks);
        collisions::n_body_collisions(&mut entities, &graph, &camera, terrain);
        // camera.show_view();
        camera.update(entities.get_entity(player).unwrap().location.position, 0.4);
        macroquad::window::next_frame().await

    }

}
