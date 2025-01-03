use macroquad::math::Vec2;
use crate::engine::graph::ExternalPointer;
use hecs::Bundle;
use derive_new::new;

#[derive(Debug, Clone, Copy, Bundle, new)]
pub struct Location {
    pub pointer: ExternalPointer,
    pub position: Vec2,
}

#[derive(Debug, Clone, Copy, Bundle, new)]
pub struct Motion {
    pub velocity: Vec2,
}

pub struct Editing;
