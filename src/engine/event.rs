#[derive(Clone, Copy, Debug)]
pub enum Event {
    Forward,
    Backward,
    Clockwise,
    CounterClockwise,

    Zoom(f32),

    SwitchColor,
    SwitchSize,
    SwitchFocus,
    PlaceBlockAtMouse,
}
