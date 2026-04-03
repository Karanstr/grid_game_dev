#[derive(Clone, Copy, Debug)]
pub enum Event {
    Forward, Backward,
    Left, Right,
    Clockwise, CounterClockwise,

    Zoom(f32),

    SwitchColor,
    SwitchSize,
    SwitchFocus,
    PlaceBlockAtMouse,
}
