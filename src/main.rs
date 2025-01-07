mod engine;
use engine::systems::io::*;
use crate::output::RenderingSystem;
use engine::utility::partition::*;
use engine::utility::blocks::*;
use engine::components::*;
use engine::graph::*;
use engine::systems::collisions::*;
use macroquad::math::*;
use hecs::World;
use derive_new::new;
use macroquad::input::*;

#[derive(new)]
struct GameData {
    entities: World,
    camera: Camera,
    graph: SparseDirectedGraph,
    blocks: BlockPalette,
}

#[macroquad::main("Window")]
async fn main() {
    macroquad::window::request_new_screen_size(512., 512.);
    let mut game_data = GameData::new(
        World::new(),
        Camera::new( AABB::new(Vec2::ZERO, Vec2::splat(2.)), 1.),
        SparseDirectedGraph::new(4),
        BlockPalette::new(),
    );
    let _ = game_data.entities.spawn((
        Location::new(
            ExternalPointer::new(InternalPointer::new(Index(0)), 2), 
            Vec2::new(2., 2.)
        ),
        Editing,
    ));
    let player = game_data.entities.spawn((
        Location::new(
            ExternalPointer::new(InternalPointer::new(Index(3)), 0), 
            Vec2::ZERO
        ),
        Velocity(Vec2::ZERO),
    ));
    loop {
        //Change this to an input module
        let player_velocity = game_data.entities.query_one_mut::<&mut Velocity>(player).unwrap();
        let speed = 0.01;
        if is_key_down(KeyCode::W) { player_velocity.0.y -= speed; }
        if is_key_down(KeyCode::S) { player_velocity.0.y += speed; }
        if is_key_down(KeyCode::A) { player_velocity.0.x -= speed; }
        if is_key_down(KeyCode::D) { player_velocity.0.x += speed; }
        input::handle_mouse_input(&mut game_data);
        RenderingSystem::draw_all(&mut game_data);
        CollisionSystem::n_body_collisions(&mut game_data);
        // game_data.camera.show_view();
        game_data.camera.update(game_data.entities.query_one_mut::<&mut Location>(player).unwrap().position, 0.4);
        macroquad::window::next_frame().await

    }

}
