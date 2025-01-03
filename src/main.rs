mod engine;
use engine::systems::io::*;
use crate::output::RenderingSystem;
use engine::utility::partition::*;
use engine::utility::blocks::*;
use engine::components::*;
use engine::graph::*;
use macroquad::math::*;
use hecs::World;
use derive_new::new;

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
        Camera::new( AABB::new(Vec2::ZERO, Vec2::splat(5.)), 0.5),
        SparseDirectedGraph::new(4),
        BlockPalette::new(),
    );
    let _ = game_data.entities.spawn((
        Location::new(
            ExternalPointer::new(InternalPointer::new(Index(2)), 2), 
            Vec2::new(2., 2.)
        ),
        Editing,
    ));
   
    loop {
        input::handle_mouse_input(&mut game_data);
        RenderingSystem::draw_all(&mut game_data);
        game_data.camera.update(game_data.camera.aabb.center(), 0.4);
        game_data.camera.show_view();
        macroquad::window::next_frame().await

    }

}
