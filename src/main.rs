mod engine;
use engine::systems::io::*;
use engine::utility::blocks::*;
use engine::graph::*;
// use engine::systems::collisions::*;
use engine::utility::partition::AABB;
use macroquad::math::*;
use derive_new::new;
use macroquad::input::*;
use output::RenderingSystem;

#[derive(Debug, Clone, Copy, new)]
pub struct Location {
    pub pointer: ExternalPointer,
    pub position: Vec2,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ID(pub u32);
#[derive(new)]
struct Entity {
    id : ID,
    location: Location,
    velocity: Vec2,
}
#[derive(new)]
//Replace Vec with dedicated entity pool struct
struct GameData {
    entities: Vec<Entity>,
    #[new(value = "0")]
    id_counter: u32,
    camera: Camera,
    graph: SparseDirectedGraph<Node>,
    blocks: BlockPalette,
}
impl GameData {
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
    let mut game_data = GameData::new(
        Vec::new(),
        Camera::new( AABB::new(Vec2::ZERO, Vec2::splat(2.)), 0.9),
        SparseDirectedGraph::new(4),
        BlockPalette::new(),
    );
    let root = game_data.graph.get_root(0, 2);
    let terrain = game_data.add_entity(
        Location::new(
            root, 
            Vec2::new(0., 0.)
        ), Vec2::ZERO
    );
    let root = game_data.graph.get_root(3, 0);
    // let player = game_data.add_entity(
    //     Location::new(
    //         root,
    //         Vec2::new(-2., 0.)
    //     ),
    //     Vec2::ZERO,
    // );
    let mut color = 0;
    loop {
        //Change this to an input module
        // let player_entity = game_data.get_mut_entity(player).unwrap();
        // let speed = 0.01;
        // if is_key_down(KeyCode::W) { player_entity.velocity.y -= speed; }
        // if is_key_down(KeyCode::S) { player_entity.velocity.y += speed; }
        // if is_key_down(KeyCode::A) { player_entity.velocity.x -= speed; }
        // if is_key_down(KeyCode::D) { player_entity.velocity.x += speed; }
        if is_key_pressed(KeyCode::V) { color += 1; color %= 4;}
        if is_key_pressed(KeyCode::P) { 
            dbg!(game_data.graph.nodes.internal_memory());
        }
        input::handle_mouse_input(&mut game_data, terrain, color);
        RenderingSystem::draw_all(&mut game_data);
        // CollisionSystem::n_body_collisions(&mut game_data);
        // game_data.camera.show_view();
        game_data.camera.update(game_data.get_entity(terrain).unwrap().location.position, 0.4);
        macroquad::window::next_frame().await

    }

}
