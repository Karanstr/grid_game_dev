use macroquad::input::*;

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Forward,
    Backward,
    Left,
    Right,
    Clockwise,
    CounterClockwise,
    Stop,
    
    Zoom(f32),

    SwitchColor,
    SwitchSize,
    PlaceBlockAtMouse,
}

#[derive(Clone, Copy)]
pub enum InputTrigger {
    Pressed,
    Down,
    Released,
}
#[derive(Clone, Copy)]
pub enum InputType {
    Keyboard(KeyCode),
    Mouse(MouseButton),
}

pub struct Input(Vec<(InputType, InputTrigger, Event)>);
impl Input {

    pub fn new() -> Self { Self(Vec::new()) }

    pub fn bind(&mut self, input: InputType, trigger: InputTrigger, event: Event) {
        self.0.push((input, trigger, event));
    }

    pub fn collect(&self, events: &mut Vec<Event>) {
        for (input, trigger, event) in &self.0 {
            if Self::should_trigger(*input, *trigger) {
                events.push(*event);
            }
        }
    }

    fn should_trigger(input: InputType, trigger: InputTrigger) -> bool {
        match (input, trigger) {
            (InputType::Keyboard(key),     InputTrigger::Pressed)  => is_key_pressed(key),
            (InputType::Keyboard(key),     InputTrigger::Down)     => is_key_down(key),
            (InputType::Keyboard(key),     InputTrigger::Released) => is_key_released(key),
            (InputType::Mouse(btn),    InputTrigger::Pressed)  => is_mouse_button_pressed(btn),
            (InputType::Mouse(btn),    InputTrigger::Down)     => is_mouse_button_down(btn),
            (InputType::Mouse(btn),    InputTrigger::Released) => is_mouse_button_released(btn),
        }
    }
}
