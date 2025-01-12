use crate::engine::graph::{SparseDirectedGraph, ExternalPointer, InternalPointer, Index};
use crate::engine::utility::partition::AABB;
use crate::engine::systems::io::Camera;
use crate::GameData;
use crate::engine::utility::partition::grid::*;
use macroquad::miniquad::window::screen_size;
use macroquad::math::{Vec2, IVec2, UVec2, BVec2};
use macroquad::color::{colors::*, Color};
use macroquad::shapes::*;
use derive_new::new;
use crate::{ID, Location};

pub mod io;
// pub mod collisions;
pub mod editing;