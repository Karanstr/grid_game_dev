use std::f32::consts::PI;
use std::collections::BinaryHeap;
use macroquad::prelude::*;
use crate::graph::{NodePointer, SparseDirectedGraph, Zorder};
pub use crate::graph::Index;
//Clean up this import stuff
mod collision_utils;
use collision_utils::*;

pub struct Object {
    pub root : NodePointer,
    pub aabs : AABS,
    pub velocity : Vec2,
    pub rotation : f32,
    pub angular_velocity : f32,
}
impl Object {
    pub fn new(root:NodePointer, position:Vec2, radius:f32) -> Self {
        Self {
            root,
            aabs : AABS::new(position, radius),
            velocity : Vec2::ZERO,
            rotation : 0.0,
            angular_velocity : 0.,
        }
    }

    pub fn effective_aabb(&self, vel_multiplier:f32) -> AABB {
        AABB::from_aabs(self.aabs).extend(self.velocity * vel_multiplier)
    }

    fn cell_length(&self, depth:u32) -> f32 {
        self.aabs.radius * 2. / 2f32.powi(depth as i32)
    }

    fn cell_top_left_corner(&self, cell:UVec2, depth:u32) -> Vec2 {
        let cell_length = self.cell_length(depth);
        cell.as_vec2() * cell_length + self.aabs.min()
    }

    //Change to relative position?
    fn coord_to_cell(&self, point:Vec2, depth:u32) -> [Option<UVec2>; 4] {
        let mut four_points = [None; 4];
        let cell_length = self.cell_length(depth);
        let offset = 0.01;
        for i in 0 .. 4 {
            let direction = Vec2::new(
                if i & 0b1 == 1 { 1. } else { -1. },
                if i & 0b10 == 0b10 { 1. } else { -1. }
            );
            let top_left = self.aabs.min();
            let cur_point = point - top_left + offset * direction;
            four_points[i] = if cur_point.clamp(Vec2::ZERO, Vec2::splat(self.aabs.radius * 2.)) == cur_point {
                Some( (cur_point / cell_length).floor().as_uvec2() )
            } else { None }
        }
        four_points
    }

    fn find_real_node(&self, world:&World, cell:UVec2, max_depth:u32) -> LimPositionData {
        let max_zorder = Zorder::from_cell(cell, max_depth);
        let (cell_pointer, real_depth) = world.graph.read(self.root, &Zorder::path(max_zorder, max_depth));
        let zorder = max_zorder >> 2 * (max_depth - real_depth);
        LimPositionData::new(cell_pointer, Zorder::to_cell(zorder, real_depth), real_depth)
    }

    //Change to relative position?
    fn get_data_at_position(&self, world:&World, position:Vec2, max_depth:u32) -> [Option<LimPositionData>; 4] {
        let max_depth_cells = self.coord_to_cell(position, max_depth);
        let mut data: [Option<LimPositionData>; 4] = [None; 4];
        for i in 0 .. 4 {
            if let Some(grid_cell) = max_depth_cells[i] {
                data[i] = Some(self.find_real_node(world, grid_cell, max_depth))
            }
        }
        data
    }

    pub fn apply_linear_force(&mut self, force:Vec2) {
        self.velocity += force;
        self.remove_neglible_vel()
    }

    pub fn apply_forward_force(&mut self, force:Vec2) {
        self.apply_linear_force(force * Vec2::from_angle(self.rotation));
    }

    fn remove_neglible_vel(&mut self) {
        let speed_min = 0.005;
        if self.velocity.x.abs() < speed_min { self.velocity.x = 0. }
        if self.velocity.y.abs() < speed_min { self.velocity.y = 0. }
    }

    pub fn apply_rotational_force(&mut self, torque:f32) {
        self.angular_velocity += torque
    }

    pub fn set_rotation(&mut self, new_rotation:f32) {
        self.rotation = new_rotation;
    }

    pub fn draw_facing(&self) {
        draw_vec_line(self.aabs.center, self.aabs.center + 10. * Vec2::new(self.rotation.cos(), self.rotation.sin()), 1., YELLOW);
    }

}


pub use vec_friendly_drawing::*;


pub struct World {
    pub blocks : BlockPalette,
    pub graph : SparseDirectedGraph,
    pub points_to_draw : Vec<(Vec2, Color, i32)>,
    pub max_depth : u32,
}
impl World {

    pub fn new(max_depth:u32) -> Self {
        Self {
            blocks : BlockPalette::new(),
            graph : SparseDirectedGraph::new(8),
            points_to_draw : Vec::new(),
            max_depth,
        }
    }

    pub fn render_cache(&mut self) {
        let mut new_points = Vec::new();
        for (point, color, time) in self.points_to_draw.iter_mut() {
            draw_centered_square(*point, 10., *color);
            let new_time = *time - 1;
            if new_time != 0 {
                new_points.push((*point, *color, new_time))
            }
        }
        self.points_to_draw = new_points;
    }

    #[allow(dead_code)]
    fn push_to_render_cache(&mut self, point:Vec2, color:Color, ticks:i32) {
        self.points_to_draw.push((point, color, ticks));
    }

    pub fn render(&self, object:&mut Object, draw_lines:bool) {
        let blocks = self.graph.dfs_leaves(object.root);
        for (zorder, depth, index) in blocks {
            match self.index_color(index) {
                Some(color) => {
                    let top_left_corner = object.cell_top_left_corner(Zorder::to_cell(zorder, depth), depth);
                    if color != BLACK {
                        draw_square(top_left_corner, object.cell_length(depth), color)
                    }
                    if draw_lines { outline_square(top_left_corner, object.cell_length(depth), 2., WHITE) }
                }
                None => { eprintln!("Failed to draw {}, unregistered block", *index) }
            }
        }
    }

    pub fn set_cell_with_mouse(&mut self, modified:&mut Object, mouse_pos:Vec2, depth:u32, index:Index) -> Result<(), String> {
        let shifted_point = mouse_pos - modified.aabs.center + modified.aabs.radius;
        if shifted_point.min_element() <= 0. || shifted_point.max_element() >= modified.aabs.radius * 2. {
            return Err("Attempting to edit beyond object domain".to_owned())
        }
        let cell = (shifted_point / modified.cell_length(depth)).ceil().as_uvec2() - 1;
        let path = Zorder::path( Zorder::from_cell(cell, depth), depth );
        if let Ok(root) = self.graph.set_node(modified.root, &path, NodePointer::new(index)) {
            modified.root = root;
            Ok(())
        } else { Err("Failed to modify cell. Likely means structure is corrupted.".to_owned()) }
    }

    fn index_collision(&self, index:Index) -> Option<OnTouch> {
        if self.blocks.blocks.len() > *index {
            Some(self.blocks.blocks[*index].collision)
        } else { None }
    }

    fn index_color(&self, index:Index) -> Option<Color> {
        if self.blocks.blocks.len() > *index {
            Some(self.blocks.blocks[*index].color)
        } else { None }
    }

    fn exposed_corners(&self, root:NodePointer, cell_zorder:u32, cell_depth:u32) -> u8 {
        let mut exposed_mask = 0b1111;
        let checks = [
            (IVec2::new(-1, 0), 0b01), //Top Left
            (IVec2::new(0, -1), 0b10),
            (IVec2::new(-1, -1), 0b11),
            (IVec2::new(1, 0), 0b00), //Top Right
            (IVec2::new(0, -1), 0b11),
            (IVec2::new(1, -1), 0b10),
            (IVec2::new(-1, 0), 0b11), //Bottom Left
            (IVec2::new(0, 1), 0b00),
            (IVec2::new(-1, 1), 0b01),
            (IVec2::new(1, 0), 0b10), //Bottom Right
            (IVec2::new(0, 1), 0b01),
            (IVec2::new(1, 1), 0b00),
        ];
        for i in 0 .. 4 {
            for j in 0 .. 3 {
                let (offset, direction) = checks[i*3 + j];
                let mut check_zorder = {
                    if let Some(zorder) = Zorder::move_cartesianly(cell_zorder, cell_depth, offset) {
                        zorder
                    } else { continue }
                };
                for _ in 0 .. self.max_depth - cell_depth {
                    check_zorder = check_zorder << 2 | direction
                }
                let path = Zorder::path(check_zorder, self.max_depth);
                let (node_pointer, _) = self.graph.read(root, &path);
                if let Some(OnTouch::Resist(walls)) = self.index_collision(node_pointer.index) {
                    if walls != BVec2::TRUE { continue }
                    exposed_mask -= 1 << i;
                    break
                }
            }
        }
        exposed_mask
    }

    fn formatted_exposed_corners(&self, object:&Object, cur_pos: Vec2, cur_vel: Vec2, obj_hit:usize) -> Vec<Particle> {
        let leaves = self.graph.dfs_leaves(object.root);
        let mut corners = Vec::new();
        for (zorder, depth, index) in leaves {
            if !matches!(self.index_collision(index).unwrap_or(OnTouch::Ignore), OnTouch::Ignore) {
                let corner_mask = self.exposed_corners(object.root, zorder, depth);
                let top_left_corner = object.cell_top_left_corner(Zorder::to_cell(zorder, depth), depth) - object.aabs.center + cur_pos;
                let cell_length = object.cell_length(depth);
                for i in 0 .. 4 {
                    if corner_mask & 1 << i != 0 {
                        corners.push(Particle::new(
                            top_left_corner + cell_length * IVec2::new(i & 1, i >> 1).as_vec2(),
                            cur_vel,
                            Configurations::from_index(i as usize),
                            obj_hit
                        ));
                    }
                }
            }
        }
        corners
    }

    pub fn render_corners(&self, object:&Object) {
        let leaves = self.graph.dfs_leaves(object.root);
        for (zorder, depth, index) in leaves {
            if !matches!(self.index_collision(index).unwrap_or(OnTouch::Ignore), OnTouch::Ignore) {
                let corner_mask = self.exposed_corners(object.root, zorder, depth);
                let top_left_corner = object.cell_top_left_corner(Zorder::to_cell(zorder, depth), depth);
                let cell_length = object.cell_length(depth);
                if corner_mask & 1 != 0 {
                    draw_vec_circle(top_left_corner, 5., YELLOW);
                }
                if corner_mask & 0b10 != 0 {
                    draw_vec_circle(top_left_corner + Vec2::new(cell_length, 0.), 5., YELLOW);
                }
                if corner_mask & 0b100 != 0 {
                    draw_vec_circle(top_left_corner + Vec2::new(0., cell_length), 5., YELLOW);
                }
                if corner_mask & 0b1000 != 0 {
                    draw_vec_circle(top_left_corner + cell_length, 5., YELLOW);
                }
            }
        }
    }

    pub fn two_way_collisions(&mut self, object1:&mut Object, object2:&mut Object, multiplier:f32) {
        if within_range(object1, object2, multiplier) {
            let mut relative_velocity= object1.velocity - object2.velocity;
            let mut modifier = Vec2::ONE;
            while relative_velocity.length_squared() != 0. {
                //Fix this at some point
                let corners = [
                    self.cull_and_fill_corners(object2, self.formatted_exposed_corners(object1, object1.aabs.center, relative_velocity, 1), multiplier),
                    self.cull_and_fill_corners(object1, self.formatted_exposed_corners(object2, object2.aabs.center, -relative_velocity, 0), multiplier)
                ];
                let corners = BinaryHeap::from(corners.concat());
                let (action, rem_rel_vel, corner_hit) = self.find_next_action([object1, object2], corners);
                object1.aabs.center += relative_velocity - rem_rel_vel * if corner_hit == 1 { 1. } else { -1. };
                relative_velocity = rem_rel_vel * if corner_hit == 1 { 1. } else { -1. };
                if let OnTouch::Resist(walls_hit) = action {
                    if walls_hit.x {
                        relative_velocity.x = 0.;
                        modifier.x = 0.
                    }
                    if walls_hit.y {
                        relative_velocity.y = 0.;
                        modifier.y = 0.;
                    }
                    object1.velocity *= modifier;
                    object2.velocity *= modifier;
                }
            }
        } else {
            object1.aabs.center += object1.velocity;
            object2.aabs.center += object2.velocity;
        }
        let drag_multiplier = -0.01;
        object1.apply_linear_force(object1.velocity * drag_multiplier);
        object2.apply_linear_force(object2.velocity * drag_multiplier);
        //make this a method?
        object1.rotation += object1.angular_velocity;
        object1.rotation %= 2.*PI;
        object1.angular_velocity = 0.;
        object2.rotation += object2.angular_velocity;
        object2.rotation %= 2.*PI;
        object2.angular_velocity = 0.;
    }

    //The self reference is mutable only so next_intersection can draw a bunch of squares
    fn find_next_action(&self, objects:[&mut Object; 2], mut corners:BinaryHeap<Particle>) -> (OnTouch, Vec2, usize) {
        let mut rel_vel_remaining = Vec2::ZERO;
        let mut action = OnTouch::Ignore;
        let mut object_hit = 0;
        while let Some(mut cur_corner) = corners.pop() {
            if cur_corner.rem_displacement.length_squared() <= rel_vel_remaining.length_squared() { break }
            let hit_point = match self.next_intersection(&cur_corner, objects[cur_corner.hitting_index], cur_corner.position_data) {
                Some(hit_point) if hit_point.ticks_to_hit < 1. => { hit_point }
                _ => { continue }
            };
            let position_data = objects[cur_corner.hitting_index].get_data_at_position(&self, hit_point.position, self.max_depth);
            cur_corner.move_to(hit_point.position, position_data);
            if let Some(data) = cur_corner.position_data {
                match self.index_collision(data.node_pointer.index) {
                    Some(OnTouch::Ignore) => { }
                    Some(OnTouch::Resist(possibly_hit_walls)) => {
                        let hit_walls = possibly_hit_walls & cur_corner.hittable_walls();
                        let checked_walls = { 
                            if hit_walls == BVec2::TRUE {
                                self.slide_check(&cur_corner, position_data)
                            } else { 
                                hit_walls 
                            }
                        };
                        if checked_walls != BVec2::FALSE {
                            action = OnTouch::Resist(
                                if checked_walls != BVec2::TRUE { checked_walls } else { cur_corner.mag_slide_check() }
                            );
                            rel_vel_remaining = cur_corner.rem_displacement;
                            object_hit = cur_corner.hitting_index;
                            continue
                        }
                    } 
                    None => { eprintln!("Attempting to touch {}, an unregistered block!", *data.node_pointer.index); }
                }
            } else { continue }
            corners.push(cur_corner);
        }
        (action, rel_vel_remaining, object_hit)
    }

    fn next_intersection(&self, particle:&Particle, object:&Object, pos_data:Option<LimPositionData>) -> Option<HitPoint> {  
        let top_left = object.aabs.min();
        let bottom_right = object.aabs.max();
        let within_bounds = BVec2::new(
            particle.position.x >= top_left.x && particle.position.x <= bottom_right.x,
            particle.position.y >= top_left.y && particle.position.y <= bottom_right.y,
        );
    
        let (cell, depth) = match pos_data {
            Some(data) => { (data.cell.as_vec2(), data.depth) }
            None => {
                let mut cell = Vec2::ZERO;
                if particle.position.x <= top_left.x {
                    if particle.rem_displacement.x > 0. { cell.x = -1. } else { return None }
                } else if particle.position.x >= bottom_right.x {
                    if particle.rem_displacement.x < 0. { cell.x = 1. } else { return None }
                }
                if particle.position.y <= top_left.y {
                    if particle.rem_displacement.y > 0. { cell.y = -1. } else { return None }
                } else if particle.position.y >= bottom_right.y {
                    if particle.rem_displacement.y < 0. { cell.y = 1. } else { return None }
                }
                (cell, 0)
            }
        };
        let quadrant = particle.rem_displacement.signum().max(Vec2::ZERO);
        let cell_length = object.cell_length(depth);
        let boundary_corner = cell * cell_length + cell_length * quadrant + top_left;
        
        let ticks = ((boundary_corner - particle.position) / particle.rem_displacement).abs();  
        let ticks_to_hit = match (within_bounds.x, within_bounds.y) {
            (false, false) => { ticks.max_element() },
            (true, false) if ticks.x == 0. => { ticks.y },
            (false, true) if ticks.y == 0. => { ticks.x },
            _ => { ticks.min_element() },
        };

        if ticks_to_hit.is_nan() || ticks_to_hit.is_infinite() { return None }
        Some(HitPoint {
            position : particle.position + particle.rem_displacement * ticks_to_hit, 
            ticks_to_hit, 
        })
    }

    fn cull_and_fill_corners(&self, hitting:&Object, unculled_corners:Vec<Particle>, multiplier:f32) -> Vec<Particle> {
        let mut corners = Vec::new();
        for corner in 0 .. unculled_corners.len() {
            if unculled_corners[corner].hittable_walls() == BVec2::FALSE { continue }
            let mut culled_corner = unculled_corners[corner].clone();
            draw_vec_circle(culled_corner.position, 5., DARKPURPLE);
            
            let hitting_aabb = hitting.effective_aabb(multiplier);
            let point_aabb = AABB::new(culled_corner.position, culled_corner.position).extend(culled_corner.rem_displacement * multiplier);
            if !hitting_aabb.intersects(point_aabb) { outline_aabb(point_aabb, 2., RED); continue }
            else { outline_aabb(point_aabb, 2., GREEN); }
            culled_corner.position_data = hitting.get_data_at_position(&self, unculled_corners[corner].position, self.max_depth)[Zorder::from_configured_direction(-unculled_corners[corner].rem_displacement, unculled_corners[corner].configuration)];
            corners.push(culled_corner);
        }
        corners
    }
   
    fn slide_check(&self, particle:&Particle, position_data:[Option<LimPositionData>; 4]) -> BVec2 {
        //Formalize this with some zorder arithmatic?
        let (x_slide_check, y_slide_check) = if particle.rem_displacement.x < 0. && particle.rem_displacement.y < 0. { //(-,-)
            (2, 1)
        } else if particle.rem_displacement.x < 0. && particle.rem_displacement.y > 0. { //(-,+)
            (0, 3)
        } else if particle.rem_displacement.x > 0. && particle.rem_displacement.y < 0. { //(+,-)
            (3, 0)
        } else { //(+,+)
            (1, 2)
        };
        let x_block_collision = if let Some(pos_data) = position_data[x_slide_check] {
            self.index_collision(pos_data.node_pointer.index).unwrap_or(OnTouch::Ignore)
        } else { OnTouch::Ignore };
        let y_block_collision = if let Some(pos_data) = position_data[y_slide_check] {
            self.index_collision(pos_data.node_pointer.index).unwrap_or(OnTouch::Ignore)
        } else { OnTouch::Ignore };
        BVec2::new(
            !matches!(y_block_collision, OnTouch::Resist(_)),
            !matches!(x_block_collision, OnTouch::Resist(_)),
        )
    }

}


pub fn within_range(object1:&Object, object2:&Object, multiplier:f32) -> bool {
    let obj1_aabb = object1.effective_aabb(multiplier);
    let obj2_aabb = object2.effective_aabb(multiplier);
    outline_aabb(obj1_aabb, 2., RED);
    outline_aabb(obj2_aabb, 2., RED);
    obj1_aabb.intersects(obj2_aabb)
}


impl Zorder {
    pub fn from_configured_direction(direction:Vec2, configuration:Configurations) -> usize {
        let clamped: Vec2 = direction.signum().max(Vec2::ZERO);
        if direction.x == 0. {
            2 * clamped.y as usize | if configuration == Configurations::TopLeft || configuration == Configurations::BottomLeft { 1 } else { 0 }
        } else if direction.y == 0. {
            clamped.x as usize | if configuration == Configurations::TopLeft || configuration == Configurations::TopRight { 2 } else { 0 }
        } else {
            2 * clamped.y as usize | clamped.x as usize
        }
    }
}


#[derive(Clone, Copy, Debug)]
pub struct AABS {
    pub center: Vec2,
    pub radius: f32,
}
#[allow(dead_code)]
impl AABS {
    pub fn new(center:Vec2, radius:f32) -> Self {
        Self { center, radius }
    }

    pub fn min(&self) -> Vec2 { 
        self.center - self.radius
    }

    pub fn max(&self) -> Vec2 {
        self.center + self.radius
    }

}
#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub top_left: Vec2,
    pub bottom_right: Vec2,
}
#[allow(dead_code)]
impl AABB {
    pub fn new(top_left:Vec2, bottom_right:Vec2) -> Self {
        Self { top_left, bottom_right }
    }

    pub fn from_aabs(aabs:AABS) -> Self {
        Self::new(aabs.min(), aabs.max())
    }

    pub fn extend(&self, distance:Vec2) -> Self {
        let direction = distance.better_sign();
        let mut new_aabb = self.clone();
         match direction.x {
            -1. => { new_aabb.top_left.x += distance.x }
            1. => { new_aabb.bottom_right.x += distance.x }
            _ => { }
        }
        match direction.y {
            -1. => { new_aabb.top_left.y += distance.y }
            1. => { new_aabb.bottom_right.y += distance.y }
            _ => { }
        }
        new_aabb
    }

    pub fn intersects(&self, other:Self) -> bool {
        (self.top_left.x < other.bottom_right.x && other.top_left.x < self.bottom_right.x) 
        && 
        (self.top_left.y < other.bottom_right.y && other.top_left.y < self.bottom_right.y)
    }

}

#[allow(dead_code)]
mod vec_friendly_drawing {
    use macroquad::prelude::*;
    use crate::AABB;

    //This is kinda silly, consider unifying square and rectangle functions, then just cope when we need to draw squares
    pub fn draw_square(top_left_corner:Vec2, length:f32, color:Color) {
        draw_rectangle(top_left_corner.x, top_left_corner.y, length, length, color);
    }

    pub fn draw_centered_square(position:Vec2, length:f32, color:Color) {
        let real_pos = position - length/2.;
        draw_rectangle(real_pos.x, real_pos.y, length, length, color);
    }

    pub fn outline_square(position:Vec2, length:f32, line_width:f32, color:Color) {
        draw_rectangle_lines(position.x, position.y, length, length, line_width, color);
    }

    pub fn outline_rectangle(position:Vec2, length:Vec2, line_width:f32, color:Color) {
        draw_rectangle_lines(position.x, position.y, length.x, length.y, line_width, color);
    }
    
    pub fn outline_centered_square(position:Vec2, length:f32, line_width:f32, color:Color) {
        let real_pos = position - length/2.;
        draw_rectangle_lines(real_pos.x, real_pos.y, length, length, line_width, color);
    }

    pub fn draw_vec_circle(position:Vec2, radius:f32, color:Color) {
        draw_circle(position.x, position.y, radius, color);
    }

    pub fn outline_circle(position:Vec2, radius:f32, line_width:f32, color:Color) {
        draw_circle_lines(position.x, position.y, radius, line_width, color);

    }

    pub fn draw_vec_line(point1:Vec2, point2:Vec2, line_width:f32, color:Color) {
        draw_line(point1.x, point1.y, point2.x, point2.y, line_width, color);
    }

    pub fn outline_aabb(aabb:AABB, line_width:f32, color:Color) {
        outline_rectangle(aabb.top_left, aabb.bottom_right - aabb.top_left, line_width, color);
    }

}


trait Vec2Extension {
    fn better_sign(&self) -> Vec2; 
}
impl Vec2Extension for Vec2 {
    fn better_sign(&self) -> Vec2 {
        Vec2::new(
            if self.x < 0. { -1. } else if self.x > 0. { 1. } else { 0. },
            if self.y < 0. { -1. } else if self.y > 0. { 1. } else { 0. },
        )
    }
}
