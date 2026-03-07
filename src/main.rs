mod engine;
mod enginev2;

use enginev2::input::*;
use enginev2::physics::*;
use enginev2::entities::*;
use glam::Vec2;
use macroquad::input::MouseButton;
use macroquad::prelude::{mouse_position, KeyCode};
use parking_lot::RwLock;
use std::f32::consts::PI;
use std::sync::Arc;
use enginev2::grid::*;

const SPEED: f32 = 0.005;
const ROTATION_SPEED: f32 = PI/512.;
const MAX_COLOR: usize = 4;
const MAX_HEIGHT: u32 = 4;

fn mouse_pos() -> Vec2 { Vec2::from(mouse_position()) }

// Figure out why BlockPalette is stupid
use crate::engine::blocks::BlockPalette;
use crate::engine::camera::Camera;

struct App {
    input: Input,
    events: Vec<Event>,

    graph: Arc<RwLock< SparseDirectedGraph<2, BasicNode2d> >>,

    entities: EntityPool,
    physics: Physics,
    
    camera: Camera,

    // I don't know where to put this
    blocks: BlockPalette,
}
impl App {
    fn intialize() -> Self {
        let mut graph = SparseDirectedGraph::new();
        // Four leaves hardcoded for now
        graph.add_leaf();
        graph.add_leaf();
        graph.add_leaf();
        graph.add_leaf();
        let wrapped_graph = Arc::new(RwLock::new(graph));
            
        let mut input = Input::new();
        set_key_binds(&mut input);

        // Either load entities here or just intialize state and have a different function for loading?

        Self {
            input,
            events: Vec::new(),

            graph: wrapped_graph,
            entities: EntityPool,
            physics: Physics::new(),
            camera: Camera::new(Vec2::ZERO, 4.),
            blocks: BlockPalette::default()
        }
    }

    async fn run(&mut self) {
        
        loop {
            let App {
                input,
                events,

                graph,
                
                entities,
                physics,
                
                camera,
                
                blocks,
            } = self;
            
            entities.draw_all(physics, camera);
            // let target = entities.get_entity(player).unwrap();
            // let old_pos = target.position(&physics);
            
            
            input.collect(events);

            handle_events(events, &mut graph.write(), entities, physics, camera);
            
            physics.tick();
            
            // We don't want to move the camera until after we've drawn all the collision debug.
            // This ensures everything lines up with the current frame.
            // camera.update(Some((old_pos, 0.4)));
            macroquad::window::next_frame().await
        }
    }

}

fn handle_events(
    events: &mut Vec<Event>,
    graph: &mut SparseDirectedGraph<2, BasicNode2d>,
    entities: &mut EntityPool,
    physics: &mut Physics,
    camera: &mut Camera,
) {
    for event in events.drain(..) {
        dbg!(event);
        match event {
            _ => println!("{:?} is unimplemented!!", event)
        }

    }
}

pub fn set_key_binds(input: &mut Input) {
    input.bind(InputType::Keyboard(KeyCode::W), InputTrigger::Down, Event::Up);
    input.bind(InputType::Keyboard(KeyCode::S), InputTrigger::Down, Event::Down);
    input.bind(InputType::Keyboard(KeyCode::A), InputTrigger::Down, Event::Left);
    input.bind(InputType::Keyboard(KeyCode::D), InputTrigger::Down, Event::Right);
    input.bind(InputType::Keyboard(KeyCode::Q), InputTrigger::Down, Event::CounterClockwise);
    input.bind(InputType::Keyboard(KeyCode::E), InputTrigger::Down, Event::Clockwise);
    input.bind(InputType::Keyboard(KeyCode::Space), InputTrigger::Pressed, Event::Stop);

    input.bind(InputType::Keyboard(KeyCode::V), InputTrigger::Pressed, Event::SwitchColor);
    input.bind(InputType::Keyboard(KeyCode::B), InputTrigger::Pressed, Event::SwitchSize);

    input.bind(InputType::Keyboard(KeyCode::Equal), InputTrigger::Down, Event::Zoom(1.02));
    input.bind(InputType::Keyboard(KeyCode::Minus), InputTrigger::Down, Event::Zoom(1./1.02));

    input.bind(InputType::Mouse(MouseButton::Left), InputTrigger::Down, Event::PlaceBlockAtMouse);
}

#[macroquad::main("")]
async fn main() {
    if !cfg!(target_arch = "wasm32") {
        // set_panic_hook();
        // init_deadlock_detection();
    }
    #[cfg(debug_assertions)]
    println!("Debug mode");
    #[cfg(not(debug_assertions))]
    println!("Release mode");
    macroquad::window::request_new_screen_size(1024., 1024.);

    let mut app = App::intialize();
    app.run().await;
}
