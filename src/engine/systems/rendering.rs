use macroquad::miniquad::window::screen_size;
use macroquad::color::{colors::*, Color};
use macroquad::math::{Vec2, BVec2};
use macroquad::shapes::*;
use crate::engine::utility::blocks::BlockPalette;
use crate::engine::graph::SparseDirectedGraph;
use crate::engine::utility::partition::AABB;
use crate::engine::components::Location;
use crate::grid::Bounds;
use derive_new::new;
use hecs::World;


pub struct Camera { 
   pub aabb:AABB,
   pub offset:Vec2,
   zoom:f32,
   zoom_modifier:f32
}
impl Camera {
    pub fn new(aabb:AABB, offset:Vec2) -> Self {
        let screen_size = Vec2::from(screen_size());
        let desired = (screen_size * 0.1).min_element();
        Self { 
            aabb, 
            offset, 
            zoom : desired, 
            zoom_modifier : 1. 
        }
    }

    pub fn update(&mut self, new_position:Vec2, smoothing:f32) {
        self.interpolate_position(new_position, smoothing);
        self.zoom = (Vec2::from(screen_size()) * 0.1).min_element();
    }

    pub fn show_view(&self) {
        self.outline_bounds(self.aabb, 2., WHITE);
    }

    pub fn interpolate_position(&mut self, position:Vec2, smoothing:f32) {
        self.aabb.move_to(self.aabb.center().lerp(position, smoothing));
    }

    pub fn interpolate_offset(&mut self, target:Vec2, smoothing:f32) {
        self.offset = self.offset.lerp(target, smoothing);
    }

    pub fn modify_zoom(&mut self, scale:f32) { self.zoom_modifier *= scale }

    pub fn zoom(&self) -> f32 { self.zoom * self.zoom_modifier }

    pub fn reset_zoom(&mut self) { self.zoom_modifier = 1. }

    pub fn expand_view(&mut self, multiplier:f32) {
        let center = self.aabb.center();
        self.aabb = self.aabb.expand(self.aabb.radius() * multiplier);
        self.aabb.move_to(center);
    }

    pub fn shrink_view(&mut self, multiplier:f32) {
        let center = self.aabb.center();
        self.aabb = self.aabb.shrink(self.aabb.radius() /2. * multiplier);
        self.aabb.move_to(center);
    }

    pub fn camera_global_offset(&self) -> Vec2 {
        self.aabb.center() + self.offset - Vec2::from(screen_size())/2./self.zoom()
    }
    pub fn draw_centered_square(&self, position:Vec2, length:f32, color:Color) {
        let real_pos = position - length/2. - self.camera_global_offset();
        draw_rectangle(real_pos.x * self.zoom(), real_pos.y * self.zoom(), length * self.zoom(), length * self.zoom(), color);
    }

    pub fn outline_centered_square(&self, position:Vec2, length:f32, line_width:f32, color:Color) {
        let real_pos = position - length/2. - self.camera_global_offset();
        draw_rectangle_lines(real_pos.x * self.zoom(), real_pos.y * self.zoom(), length * self.zoom(), length * self.zoom(), line_width, color);
    }

    pub fn draw_vec_rectangle(&self, position:Vec2, length:Vec2, color:Color) {
        let pos = position - self.camera_global_offset();
        draw_rectangle(pos.x * self.zoom(), pos.y * self.zoom(), length.x * self.zoom(), length.y * self.zoom(), color);
    }

    pub fn outline_vec_rectangle(&self, position:Vec2, length:Vec2, line_width:f32, color:Color) {
        let pos = position - self.camera_global_offset();
        draw_rectangle_lines(pos.x * self.zoom(), pos.y * self.zoom(), length.x * self.zoom(), length.y * self.zoom(), line_width, color);
    }
    
    pub fn draw_vec_circle(&self, position:Vec2, radius:f32, color:Color) {
        let pos = position - self.camera_global_offset();
        draw_circle(pos.x * self.zoom(), pos.y * self.zoom(), radius * self.zoom(), color);
    }

    pub fn outline_vec_circle(&self, position:Vec2, radius:f32, line_width:f32, color:Color) {
        let pos = position - self.camera_global_offset();
        draw_circle_lines(pos.x * self.zoom(), pos.y * self.zoom(), radius * self.zoom(), line_width, color);
    }

    pub fn draw_vec_line(&self, point1:Vec2, point2:Vec2, line_width:f32, color:Color) {
        let p1 = point1 - self.camera_global_offset();
        let p2 = point2 - self.camera_global_offset();
        draw_line(p1.x * self.zoom(), p1.y * self.zoom(), p2.x * self.zoom(), p2.y * self.zoom(), line_width, color);
    }

    pub fn draw_bounds(&self, bounds:AABB, color:Color) {
        self.draw_vec_rectangle(bounds.min(), bounds.max() - bounds.min(), color);
    }

    pub fn outline_bounds(&self, bounds:AABB, line_width:f32, color:Color) {
        self.outline_vec_rectangle(bounds.min(), bounds.max() - bounds.min(), line_width, color);
    }


}


#[derive(new)]
pub struct RenderingSystem(pub Camera);

impl RenderingSystem {

   pub fn draw_all(&self, world: &mut World, graph: &SparseDirectedGraph, blocks: &BlockPalette) {
      let entities = world.query_mut::<&Location>();
      for (_, location) in entities {
         if self.0.aabb.intersects(Bounds::aabb(location.position, location.pointer.height)) == BVec2::TRUE {
            self.draw(location, graph, blocks);
         }
      }
   }

   pub fn draw(&self, location: &Location, graph: &SparseDirectedGraph, blocks: &BlockPalette) {
      let object_top_left = location.position - Bounds::cell_length(location.pointer.height) / 2.;
      let leaves = graph.dfs_leaves(location.pointer);
      for leaf in leaves {
         let color = blocks.blocks[*leaf.pointer.pointer.index].color; 
         if 0 != *leaf.pointer.pointer.index {
            self.0.draw_vec_rectangle(
               object_top_left + Bounds::top_left_corner(leaf.cell, leaf.pointer.height),
               Bounds::cell_length(leaf.pointer.height),
               color
            );
         }
      }
   }

}