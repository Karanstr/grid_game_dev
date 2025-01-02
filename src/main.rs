mod engine;
use engine::systems::rendering::*;
use engine::utility::partition::*;
use engine::utility::blocks::*;
use engine::components::*;
use engine::graph::*;
use macroquad::window::{request_new_screen_size, next_frame};
use macroquad::math::*;
use hecs::World;

#[macroquad::main("Window")]
async fn main() {
    let size = Vec2::splat(256.);
    request_new_screen_size(size.x*2., size.y*2.);
    let camera = Camera::new(AABB::new(size, Vec2::splat(1.)), Vec2::ZERO);
    let rendering_system = RenderingSystem::new(camera);

    let graph = SparseDirectedGraph::new(4);
    let block_palette = BlockPalette::new(); 
    let mut world = World::new();
    let _ = world.spawn((
        Location::new(ExternalPointer::new(InternalPointer::new(Index(1)), 0), size),
    ));
   
    loop {
        
        rendering_system.0.show_view();
        rendering_system.draw_all(&mut world, &graph, &block_palette);
        next_frame().await
    }

}
