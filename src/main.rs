mod engine;
use engine::{
    input::*,
    physics::*,
    entities::*,
    camera::Camera,
    grid::*,
    event::Event
};

use glam::Vec2;
use macroquad::input::{KeyCode, MouseButton, mouse_position};
use parking_lot::RwLock;
use rapier2d::math::Pose2;
use std::f32::consts::PI;
use std::sync::Arc;

const SPEED: f32 = 0.1;
const ROTATION_SPEED: f32 = PI/128.;
const MAX_COLOR: usize = 4;
const MAX_HEIGHT: u32 = 4;

pub type GRAPH = Arc<RwLock< SparseDirectedGraph<2, BasicNode2d>>>;

struct App {
    input: Input,
    events: Vec<Event>,

    entities: EntityPool,
    physics: Physics,

    camera: Camera,

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

        let mut entities = EntityPool::new(wrapped_graph);
        let mut physics = Physics::default();
        let (head, height) = {
            let mut graph = entities.graph.write();
            let head = graph.get_root(1);
            let height = 3;
            (head, height)
        };

        entities.add(
            DagPointer::new(head, height),
            Pose2::new(Vec2::ZERO, 0.),
            &mut physics
        );

        let mut input = Input::new();
        set_key_binds(&mut input);

        // Initial loading here, likely by passing a serialized_state in

        Self {
            input,
            events: Vec::new(),

            entities,
            physics,

            camera: Camera::new(Vec2::ZERO, 4.),
        }
    }

    async fn run(&mut self) {
        loop {
            let App {
                input,
                events,

                entities,
                physics,
                
                camera,
            } = self;

            camera.update_screensize();

            input.collect(events);
            handle_events(events, entities, physics, camera);
            
            physics.tick();

            entities.draw_all(physics, camera);
            
            macroquad::window::next_frame().await
        }
    }

}

fn handle_events(events: &mut Vec<Event>, entities: &mut EntityPool, physics: &mut Physics, camera: &mut Camera) {
    for event in events.drain(..) { match event {
        Event::Clockwise => {
            entities.get(0).unwrap().rotate_by(ROTATION_SPEED, physics);
        }
        Event::CounterClockwise => {
            entities.get(0).unwrap().rotate_by(-ROTATION_SPEED, physics);
        }
        Event::Forward => {
            entities.get(0).unwrap().move_wrt_rotation(Vec2::X * SPEED, physics);
        }
        Event::Backward => {
            entities.get(0).unwrap().move_wrt_rotation(Vec2::NEG_X * SPEED, physics);
        }
        Event::Zoom(zoom) => {
            camera.zoom_by(zoom);
        }
        Event::PlaceBlockAtMouse => {
            let mouse_pos = Vec2::from(mouse_position());
            let world_pos = camera.screen_to_world(mouse_pos);
            entities.get_mut(0).unwrap().set_block(physics, world_pos, DagPointer::new(0, 0));
        }
        _ => println!("{:?} is unimplemented!!", event)
    } }
}

pub fn set_key_binds(input: &mut Input) {
    input.bind(InputType::Keyboard(KeyCode::W), InputTrigger::Down, Event::Forward);
    input.bind(InputType::Keyboard(KeyCode::S), InputTrigger::Down, Event::Backward);
    input.bind(InputType::Keyboard(KeyCode::A), InputTrigger::Down, Event::CounterClockwise);
    input.bind(InputType::Keyboard(KeyCode::D), InputTrigger::Down, Event::Clockwise);
    // input.bind(InputType::Keyboard(KeyCode::Space), InputTrigger::Pressed, Event::Stop);

    // input.bind(InputType::Keyboard(KeyCode::V), InputTrigger::Pressed, Event::SwitchColor);
    // input.bind(InputType::Keyboard(KeyCode::B), InputTrigger::Pressed, Event::SwitchSize);

    input.bind(InputType::Keyboard(KeyCode::Equal), InputTrigger::Down, Event::Zoom(1.02));
    input.bind(InputType::Keyboard(KeyCode::Minus), InputTrigger::Down, Event::Zoom(1./1.02));

    input.bind(InputType::Mouse(MouseButton::Left), InputTrigger::Down, Event::PlaceBlockAtMouse);
}

#[macroquad::main("")]
async fn main() {
    #[cfg(debug_assertions)]
    println!("Debug mode");
    #[cfg(not(debug_assertions))]
    println!("Release mode");
    macroquad::window::request_new_screen_size(1024., 1024.);

    let mut app = App::intialize();
    app.run().await;
}
