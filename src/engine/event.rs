#[derive(Clone, Copy, Debug)]
pub enum Event {
    Forward,
    Backward,
    Clockwise,
    CounterClockwise,
    Stop,
    
    Zoom(f32),

    SwitchColor,
    SwitchSize,
    PlaceBlockAtMouse,
}
