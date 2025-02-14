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
}
#[allow(dead_code)]
impl InputHandler {
    pub fn new() -> Self {
        Self {
            bindings: HashMap::new(),
            next_id: 0,
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

    /// Returns success
    pub fn remove(&mut self, id: BindingId) -> bool {
        self.bindings.remove(&id).is_some()
    }

    /// Returns success and current value
    pub fn toggle(&mut self, id: BindingId) -> Option<bool> {
        let binding = self.bindings.get_mut(&id)?;
        binding.enabled = !binding.enabled;
        Some(binding.enabled)
    }

    /// Returns success and old value
    pub fn enable(&mut self, id: BindingId) -> Option<bool> {
        let binding = self.bindings.get_mut(&id)?;
        let old_binding = binding.enabled;
        binding.enabled = true;
        Some(old_binding)
    }

    /// Returns success and old value
    pub fn disable(&mut self, id: BindingId) -> Option<bool> {
        let binding = self.bindings.get_mut(&id)?;
        let old_binding = binding.enabled;
        binding.enabled = false;
        Some(old_binding)
    }

    fn should_trigger(input: InputType, trigger: InputTrigger) -> bool {
        match input {
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
            }
        }
    }

    /// Loops through all bindings and executes actions
    pub fn handle(&mut self) {
        for binding in self.bindings.values_mut() {
            if binding.enabled && Self::should_trigger(binding.input, binding.trigger) {
                (binding.action)();
            }
        }
    }
    
}
