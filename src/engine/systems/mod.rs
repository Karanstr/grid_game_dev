use crate::engine::graph::{SparseDirectedGraph, ExternalPointer, InternalPointer, Index};
use crate::engine::components::{Location, Editing};
use crate::engine::utility::partition::AABB;
use derive_new::new;
use crate::grid::*;
use crate::GameData;
use hecs::{World, Entity};
use macroquad::miniquad::window::screen_size;
use macroquad::math::{Vec2, IVec2, UVec2, BVec2};
use macroquad::color::{colors::*, Color};
use macroquad::shapes::*;

pub mod io;
pub mod collisions;
pub mod editing;