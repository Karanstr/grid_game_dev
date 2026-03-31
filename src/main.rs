mod engine;
use engine::{
    input::*,
    physics::*,
    entities::*,
    camera::Camera,
    grid::dim2::*,
    event::Event
};

use glam::Vec2;
use macroquad::input::{KeyCode, MouseButton, mouse_position, mouse_position_local};
use parking_lot::RwLock;
use rapier2d::math::Pose2;
use std::f32::consts::PI;
use std::sync::Arc;

const SPEED: f32 = 0.1;
const ROTATION_SPEED: f32 = PI/128.;
const MOD_COLOR: u32 = 4;
const MOD_HEIGHT: u32 = 4;

pub type GRAPH = Arc<RwLock< Graph2D<BasicNode2d> >>;

struct App {
    input: Input,
    events: Vec<Event>,
    other_data: OtherData,

    entities: EntityPool,
    physics: Physics,

    camera: Camera,
}
impl App {
    fn intialize() -> Self {
        let mut graph = Graph2D::new();
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
            Pose2::new(Vec2::new(0., -20.), 0.),
            &mut physics
        );

        let (head, height) = {
            let mut graph = entities.graph.write();
            let head = graph.get_root(2);
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
            other_data: OtherData::default(),

            entities,
            physics,

            camera: Camera::new(Vec2::ZERO, 12.),
        }
    }

    async fn run(&mut self) {
        loop {
            let App {
                input,
                events,
                other_data,

                entities,
                physics,
                
                camera,
            } = self;

            camera.update_screensize();

            input.collect(events);
            handle_events(events, entities, physics, camera, other_data);
            
            physics.tick();

            entities.draw_all(physics, camera);

            let entity = entities.get(other_data.entity).unwrap().rb_handle;
            let position = physics.rigid_bodies.get(entity).unwrap().center_of_mass();
            let mouse_ratio = mouse_position_local();
            camera.follow(position + Vec2::new(mouse_ratio.x, mouse_ratio.y) * 2.5, 0.2);
            
            macroquad::window::next_frame().await
        }
    }

}

fn handle_events(events: &mut Vec<Event>, entities: &mut EntityPool, physics: &mut Physics, camera: &mut Camera, data: &mut OtherData) {
    for event in events.drain(..) { match event {
        Event::Clockwise => {
            entities.get(data.entity).unwrap().rotate_by(ROTATION_SPEED, physics);
        }
        Event::CounterClockwise => {
            entities.get(data.entity).unwrap().rotate_by(-ROTATION_SPEED, physics);
        }
        Event::Forward => {
            entities.get(data.entity).unwrap().move_wrt_rotation(Vec2::X * SPEED, physics);
        }
        Event::Backward => {
            entities.get(data.entity).unwrap().move_wrt_rotation(Vec2::NEG_X * SPEED, physics);
        }
        Event::Zoom(zoom) => {
            camera.zoom_by(zoom);
        }
        Event::SwitchColor => {
            data.color = (data.color + 1) % MOD_COLOR
        }
        Event::SwitchSize => {
            data.height = (data.height + 1) % MOD_HEIGHT
        }
        Event::SwitchFocus => {
            data.entity = (data.entity + 1) % entities.len()
        }
        Event::PlaceBlockAtMouse => {
            let mouse_pos = Vec2::from(mouse_position());
            let world_pos = camera.screen_to_world(mouse_pos);
            entities.get_mut(data.entity).unwrap().set_block(
                physics,
                world_pos,
                DagPointer::new(data.color, data.height)
            );
        }
        // _ => println!("{:?} is unimplemented!!", event)
    } }
}

// I don't love this but I need something for now
struct OtherData {
    color: u32,
    height: u32,
    entity: usize,
}
impl Default for OtherData {
    fn default() -> Self {
        Self {
            color: 0,
            height: 0,
            entity: 1,
        }
    }
}

pub fn set_key_binds(input: &mut Input) {
    input.bind(InputType::Keyboard(KeyCode::W), InputTrigger::Down, Event::Forward);
    input.bind(InputType::Keyboard(KeyCode::S), InputTrigger::Down, Event::Backward);
    input.bind(InputType::Keyboard(KeyCode::A), InputTrigger::Down, Event::CounterClockwise);
    input.bind(InputType::Keyboard(KeyCode::D), InputTrigger::Down, Event::Clockwise);

    input.bind(InputType::Keyboard(KeyCode::V), InputTrigger::Pressed, Event::SwitchColor);
    input.bind(InputType::Keyboard(KeyCode::B), InputTrigger::Pressed, Event::SwitchSize);
    input.bind(InputType::Keyboard(KeyCode::C), InputTrigger::Pressed, Event::SwitchFocus);

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
