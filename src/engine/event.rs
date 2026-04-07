#[derive(Clone, Copy, Debug)]
pub enum Event {
    Dbg, // I want clippy to shut up about unreachable patterns

    Forward, Backward,
    Left, Right,
    Clockwise, CounterClockwise,

    Zoom(f32),

    SwitchColor,
    SwitchSize,
    SwitchFocus,
    PlaceBlockAtMouse,
}
