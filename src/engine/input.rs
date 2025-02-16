use std::collections::HashMap;
use macroquad::input::*;

pub type BindingId = usize;

#[allow(dead_code)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputTrigger {
    Pressed,
    Down,
    Released,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum InputType {
    Keyboard(KeyCode),
    Mouse(MouseButton),
}

pub struct InputBinding {
    input: InputType,
    trigger: InputTrigger,
    action: Box<dyn FnMut()>,
    enabled: bool,
}

pub struct InputHandler {
    bindings: HashMap<BindingId, InputBinding>,
    next_id: BindingId,
    injected_events: Vec<(InputType, InputTrigger)>,
}
#[allow(dead_code)]
impl InputHandler {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            next_id: 0,
            injected_events: Vec::new(),
        }
    }

    pub fn bind_key<F>(&mut self, key: KeyCode, trigger: InputTrigger, action: F) -> BindingId 
    where
        F: FnMut() + 'static,
    {
        self.bind(InputType::Keyboard(key), trigger, action)
    }

    pub fn bind_mouse<F>(&mut self, button: MouseButton, trigger: InputTrigger, action: F) -> BindingId 
    where
        F: FnMut() + 'static,
    {
        self.bind(InputType::Mouse(button), trigger, action)
    }

    fn bind<F>(&mut self, input: InputType, trigger: InputTrigger, action: F) -> BindingId 
    where
        F: FnMut() + 'static,
    {
        let id = self.next_id;
        self.next_id += 1;
        
        self.bindings.insert(id, InputBinding {
            input,
            trigger,
            action: Box::new(action),
            enabled: true,
        });
        
        id
    }

    /// Injects an input which will be processed next frame
    pub fn inject(&mut self, input: InputType, trigger: InputTrigger) {
        self.injected_events.push((input, trigger));
    }

    /// Returns success
    pub fn remove(&mut self, id: BindingId) -> bool {
        self.bindings.remove(&id).is_some()
    }

    /// Returns current value on success
    pub fn toggle(&mut self, id: BindingId) -> Option<bool> {
        let binding = self.bindings.get_mut(&id)?;
        binding.enabled = !binding.enabled;
        Some(binding.enabled)
    }

    /// Returns old value on success
    pub fn enable(&mut self, id: BindingId) -> Option<bool> {
        let binding = self.bindings.get_mut(&id)?;
        let old_binding = binding.enabled;
        binding.enabled = true;
        Some(old_binding)
    }

    /// Returns old value on success
    pub fn disable(&mut self, id: BindingId) -> Option<bool> {
        let binding = self.bindings.get_mut(&id)?;
        let old_binding = binding.enabled;
        binding.enabled = false;
        Some(old_binding)
    }

    fn should_trigger(input: InputType, trigger: InputTrigger, injected: &[(InputType, InputTrigger)]) -> bool {
        let user = match input {
            InputType::Keyboard(key) => {
                match trigger {
                    InputTrigger::Pressed => is_key_pressed(key),
                    InputTrigger::Down => is_key_down(key),
                    InputTrigger::Released => is_key_released(key),
                }
            },
            InputType::Mouse(button) => {
                match trigger {
                    InputTrigger::Pressed => is_mouse_button_pressed(button),
                    InputTrigger::Down => is_mouse_button_down(button),
                    InputTrigger::Released => is_mouse_button_released(button),
                }
            },
        };
        if user || injected.contains(&(input, trigger)) { true }
        else { false }
    }

    /// Loops through all bindings and executes actions
    pub fn handle(&mut self) {
        // This is gross but a good temporary solution
        let injected = std::mem::take(&mut self.injected_events);
        for binding in self.bindings.values_mut() {
            if binding.enabled && Self::should_trigger(binding.input, binding.trigger, &injected) {
                (binding.action)();
            }
        }
    }
}
