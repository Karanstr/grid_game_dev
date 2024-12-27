use std::f32::consts::PI;
use std::collections::BinaryHeap;
use std::cmp::Reverse;
use macroquad::prelude::*;
use crate::graph::{NodePointer, SparseDirectedGraph, Zorder};
pub use crate::graph::Index;
use crate::drawing_camera::Camera;
use crate::utilities::*;
mod collision_utils;
use collision_utils::*;

pub struct Object {
    pub aabb : AABB,
    pub root : NodePointer,
    pub velocity : Vec2,
    pub rotation : f32,
    pub angular_velocity : f32,
}
impl Object {
    pub fn new(root:NodePointer, position:Vec2, radius:f32) -> Self {
        Self {
            aabb : AABB::new(position, Vec2::splat(radius)),
            root,
            velocity : Vec2::ZERO,
            rotation : 0.0,
            angular_velocity : 0.,
        }
    }

    pub fn effective_aabb(&self, vel_multiplier:f32) -> AABB {
        self.aabb.expand(self.velocity * vel_multiplier)
    }

    fn cell_length(&self, depth:u32) -> f32 {
        self.aabb.radius().x * 2. / 2f32.powi(depth as i32)
    }

    fn cell_top_left_corner(&self, cell:UVec2, depth:u32) -> Vec2 {
        let cell_length = self.cell_length(depth);
        cell.as_vec2() * cell_length + self.aabb.min()
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
            let cur_point = point - self.aabb.min() + offset * direction;
            if cur_point.clamp(Vec2::ZERO, self.aabb.radius() * 2.) == cur_point {
                four_points[i] = Some( (cur_point / cell_length).floor().as_uvec2() )
            }
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

    pub fn update_rotation(&mut self) {
        self.rotation += self.angular_velocity;
        self.rotation %= 2.*PI;
        self.angular_velocity = 0.;
    }

    pub fn set_rotation(&mut self, new_rotation:f32) {
        self.rotation = new_rotation;
    }

    pub fn draw_facing(&self, camera:&Camera) {
        camera.draw_vec_line(self.aabb.center(), self.aabb.center() + 10. * Vec2::new(self.rotation.cos(), self.rotation.sin()), 1., YELLOW);
    }

}

pub struct World {
    pub graph : SparseDirectedGraph,
    pub blocks : BlockPalette,
    pub max_depth : u32,
    pub camera : Camera,
}
impl World {
    pub fn new(max_depth:u32, camera:Camera) -> Self {
        Self {
            graph : SparseDirectedGraph::new(8),
            blocks : BlockPalette::new(),
            max_depth,
            camera
        }
    }

    pub fn render(&self, object:&mut Object, draw_lines:bool) {
        let blocks = self.graph.dfs_leaves(object.root);
        for (zorder, depth, index) in blocks {
            match self.index_color(index) {
                Some(color) => {
                    let top_left_corner = object.cell_top_left_corner(Zorder::to_cell(zorder, depth), depth);
                    if color != BLACK {
                        self.camera.draw_vec_rectangle(top_left_corner, Vec2::splat(object.cell_length(depth)), color);
                    }
                    if draw_lines { self.camera.outline_vec_rectangle(top_left_corner, Vec2::splat(object.cell_length(depth)), 2., WHITE) }
                }
                None => { eprintln!("Failed to draw {}, unregistered block", *index) }
            }
        }
    }

    pub fn set_cell_with_mouse(&mut self, modified:&mut Object, mouse_pos:Vec2, depth:u32, index:Index) -> Result<(), String> {
        let shifted_point = mouse_pos/self.camera.zoom - modified.aabb.min() + self.camera.camera_global_offset();
        if shifted_point.min_element() <= 0. || shifted_point.max_element() >= modified.aabb.radius().x * 2. {
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

    //Make this not bad?
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

    fn formatted_exposed_corners(&self, object_within:&Object, cur_pos: Vec2, ticks_into_projection:f32, owner:usize, hitting:usize) -> Vec<Particle> {
        let leaves = self.graph.dfs_leaves(object_within.root);
        let mut corners = Vec::new();
        for (zorder, depth, index) in leaves {
            if !matches!(self.index_collision(index).unwrap_or(OnTouch::Ignore), OnTouch::Ignore) {
                let corner_mask = self.exposed_corners(object_within.root, zorder, depth);
                let top_left_corner = object_within.cell_top_left_corner(Zorder::to_cell(zorder, depth), depth) - object_within.aabb.center() + cur_pos;
                let cell_length = object_within.cell_length(depth);
                for i in 0 .. 4 {
                    if corner_mask & 1 << i != 0 {
                        corners.push(Particle::new(
                            top_left_corner + cell_length * IVec2::new(i & 1, i >> 1).as_vec2(),
                            ticks_into_projection,
                            Configurations::from_index(i as usize),
                            owner,
                            hitting,
                        ));
                    }
                }
            }
        }
        corners
    }

    fn cull_and_fill_corners(&self, hitting:&Object, mut unculled_corners:Vec<Particle>, velocity:Vec2, multiplier:f32) -> Vec<Reverse<Particle>> {
        let mut corners = Vec::new();
        while let Some(mut corner) = unculled_corners.pop() {
            if hittable_walls(velocity, corner.configuration) == BVec2::FALSE { continue }
            let hitting_aabb = hitting.effective_aabb(multiplier);
            let point_aabb = AABB::new(corner.position, Vec2::ZERO).expand( velocity * multiplier);
            if hitting_aabb.intersects(point_aabb) != BVec2::TRUE { self.camera.outline_bounds(point_aabb, 2., RED); continue }
            else { self.camera.outline_bounds(point_aabb, 2., GREEN); }
            corner.position_data = hitting.get_data_at_position(&self, corner.position, self.max_depth)[Zorder::from_configured_direction(-velocity, corner.configuration)];
            corners.push(Reverse(corner));
        }
        corners
    }
   
    //Clean this up and make it n-body compatible
    fn get_corners(&self, object1:&Object, object2:&Object, ticks_into_projection:f32, multiplier:f32, obj1_index:usize, obj2_index:usize) -> BinaryHeap<Reverse<Particle>> {
        let relative_velocity = object1.velocity - object2.velocity;
        let corners = [
            self.cull_and_fill_corners(object2, self.formatted_exposed_corners(object1, object1.aabb.center(), ticks_into_projection, obj1_index, obj2_index), relative_velocity, multiplier),
            self.cull_and_fill_corners(object1, self.formatted_exposed_corners(object2, object2.aabb.center(), ticks_into_projection, obj2_index, obj1_index), -relative_velocity, multiplier)
        ];
        BinaryHeap::from(corners.concat())
    }

    pub fn n_body_collisions(&self, mut objects:Vec<&mut Object>, multiplier:f32) {
        let mut ticks_into_projection = 0.;
        loop {
            let tick_max = 1. - ticks_into_projection;
            let mut corners = BinaryHeap::new();
            for i in 0 .. objects.len() {
                for j in i + 1 .. objects.len() { 
                    if within_range(objects[i], objects[j], multiplier, &self.camera) {
                        let (obj1_index, obj2_index) = (i, j);
                        corners.extend(self.get_corners(objects[i], objects[j], 0., multiplier, obj1_index, obj2_index));
                    }
                }
            }
            let Some((action, ticks_at_hit, (owner, hitting))) = self.find_next_action(&objects, corners, tick_max) else {
                //No collision, move objects their remaining distance
                for object in objects.iter_mut() {
                    object.aabb.move_by(object.velocity * tick_max);
                }
                break
            };
            ticks_into_projection += ticks_at_hit;
            for object in objects.iter_mut() {
                object.aabb.move_by(object.velocity * ticks_at_hit);
            }
            if let OnTouch::Resist(walls_hit) = action {
                let energy_conserved = 1.0;
                let relative_velocity = objects[owner].velocity - objects[hitting].velocity;
                let impulse = -(1. + energy_conserved) * relative_velocity / 2.;
                if walls_hit.x {
                    objects[owner].velocity.x += impulse.x;
                    objects[hitting].velocity.x -= impulse.x;
                    // objects[1].velocity.x = 0.;
                }
                if walls_hit.y {
                    objects[owner].velocity.y += impulse.y;
                    objects[hitting].velocity.y -= impulse.y;
                    // objects[1].velocity.y = 0.;
                }
            }
        }
        
        let drag_multiplier = -0.01;
        for object in objects {
            object.apply_linear_force(object.velocity * drag_multiplier);
            object.update_rotation();
        }
    }

    //Replace this return type with a struct
    fn find_next_action(&self, objects:&Vec<&mut Object>, mut corners:BinaryHeap<Reverse<Particle>>, tick_max:f32) -> Option<(OnTouch, f32, (usize, usize))> {
        let mut action = OnTouch::Ignore;
        let mut ticks_to_hit = tick_max;
        let mut hit = false;
        let mut col_owner = 0;
        let mut col_hitting = 0;
        while let Some(mut cur_corner) = corners.pop().map(|x| x.0) {
            if cur_corner.ticks_into_projection >= ticks_to_hit { break }
            let (owner, hitting) = cur_corner.rel_objects;
            let relative_velocity = objects[owner].velocity - objects[hitting].velocity;
            let Some(hit_point) = self.next_intersection(cur_corner.position, relative_velocity, cur_corner.position_data, objects[hitting]) else { continue };
            cur_corner.ticks_into_projection += hit_point.ticks_to_hit;
            if cur_corner.ticks_into_projection >= ticks_to_hit { continue }
            cur_corner.position = hit_point.position;
            let position_data = objects[hitting].get_data_at_position(&self, cur_corner.position, self.max_depth);
            cur_corner.position_data = position_data[Zorder::from_configured_direction(relative_velocity, cur_corner.configuration)];
            let Some(data) = cur_corner.position_data else { continue };
            match self.index_collision(data.node_pointer.index) {
                Some(OnTouch::Ignore) => { }
                Some(OnTouch::Resist(possibly_hit_walls)) => {
                    if let Some(hit_walls) = self.determine_walls_hit(possibly_hit_walls, relative_velocity, cur_corner.configuration, position_data) {
                        hit = true;
                        col_owner = owner;
                        col_hitting = hitting;
                        action = OnTouch::Resist(hit_walls);
                        ticks_to_hit = cur_corner.ticks_into_projection;
                        continue
                    }
                } 
                None => { eprintln!("Attempting to touch {}, an unregistered block!", *data.node_pointer.index); }
            }
            corners.push(Reverse(cur_corner));
        }
        if hit { Some((action, ticks_to_hit, (col_owner, col_hitting))) }
        else { None }
    }

    fn determine_walls_hit(&self, possibly_hit_walls:BVec2, velocity:Vec2, configuration:Configurations, position_data:[Option<LimPositionData>; 4]) -> Option<BVec2> {
        let hit_walls = {
            let potential_hits = possibly_hit_walls & hittable_walls(velocity, configuration);
            if potential_hits == BVec2::TRUE {
                self.slide_check(velocity, position_data)
            } else {
                potential_hits
            }
        };
        match hit_walls {
            BVec2::TRUE => { Some(mag_slide_check(velocity)) }
            BVec2::FALSE => { None }
            _ => { Some(hit_walls) }
        }
    }

    fn next_intersection(&self, position:Vec2, velocity:Vec2, position_data:Option<LimPositionData>, hitting:&Object) -> Option<HitPoint> {
        let top_left = hitting.aabb.min();
        let bottom_right = hitting.aabb.max();
        //Replace with aabb check?
        let within_bounds = hitting.aabb.contains(position);
        let (cell, depth) = match position_data {
            Some(data) => { (data.cell.as_vec2(), data.depth) }
            None => {
                let mut cell = Vec2::ZERO;
                if position.x <= top_left.x {
                    if velocity.x > 0. { cell.x = -1. } else { return None }
                } else if position.x >= bottom_right.x {
                    if velocity.x < 0. { cell.x = 1. } else { return None }
                }
                if position.y <= top_left.y {
                    if velocity.y > 0. { cell.y = -1. } else { return None }
                } else if position.y >= bottom_right.y {
                    if velocity.y < 0. { cell.y = 1. } else { return None }
                }
                (cell, 0)
            }
        };
        let quadrant = velocity.signum().max(Vec2::ZERO);
        let cell_length = hitting.cell_length(depth);
        let boundary_corner = top_left + cell * cell_length + cell_length * quadrant;
        
        let ticks = ((boundary_corner - position) / velocity).abs(); 
        let ticks_to_hit = match (within_bounds.x, within_bounds.y) {
            (false, false) => { ticks.max_element() },
            (true, false) if ticks.x == 0. => { ticks.y },
            (false, true) if ticks.y == 0. => { ticks.x },
            _ => { ticks.min_element() },
        };
        if ticks_to_hit.is_nan() || ticks_to_hit.is_infinite() { return None }
        Some(HitPoint {
            position : position + velocity * ticks_to_hit, 
            ticks_to_hit, 
        })
    }

    fn slide_check(&self, velocity:Vec2, position_data:[Option<LimPositionData>; 4]) -> BVec2 {
        //Formalize this with some zorder arithmatic?
        let (x_slide_check, y_slide_check) = if velocity.x < 0. && velocity.y < 0. { //(-,-)
            (2, 1)
        } else if velocity.x < 0. && velocity.y > 0. { //(-,+)
            (0, 3)
        } else if velocity.x > 0. && velocity.y < 0. { //(+,-)
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
        let result = BVec2::new(
            matches!(y_block_collision, OnTouch::Ignore),
            matches!(x_block_collision, OnTouch::Ignore),
        );
        result
    }

}

//Figure out where to put these
pub fn hittable_walls(velocity:Vec2, configuration:Configurations) -> BVec2 {
    let (x_check, y_check) = match configuration {
        Configurations::TopLeft => {
            (velocity.x < 0., velocity.y < 0.)
        }
        Configurations::TopRight => {
            (velocity.x > 0., velocity.y < 0.)
        }
        Configurations::BottomLeft => {
            (velocity.x < 0., velocity.y > 0.)
        }
        Configurations::BottomRight => {
            (velocity.x > 0., velocity.y > 0.)
        }
    };
    BVec2::new(x_check, y_check)
}

pub fn mag_slide_check(velocity:Vec2) -> BVec2 {
    let abs_vel = velocity.abs();
    if abs_vel.y < abs_vel.x { 
        BVec2::new(false, true)
    } else if abs_vel.x < abs_vel.y {
        BVec2::new(true, false)
    } else {
        BVec2::TRUE
    }
}

pub fn within_range(object1:&Object, object2:&Object, multiplier:f32, camera:&Camera) -> bool {
    let obj1_aabb = object1.effective_aabb(multiplier);
    let obj2_aabb = object2.effective_aabb(multiplier);
    camera.outline_bounds(obj1_aabb, 2., RED);
    camera.outline_bounds(obj2_aabb, 2., RED);
    obj1_aabb.intersects(obj2_aabb) == BVec2::TRUE
}


impl Zorder {
    pub fn from_configured_direction(direction:Vec2, configuration:Configurations) -> usize {
        let clamped: Vec2 = direction.signum().max(Vec2::ZERO);
        if direction == Vec2::ZERO { dbg!("AHHH"); }
        if direction.x == 0. {
            2 * clamped.y as usize | if configuration == Configurations::TopLeft || configuration == Configurations::BottomLeft { 1 } else { 0 }
        } else if direction.y == 0. {
            clamped.x as usize | 2 * if configuration == Configurations::TopLeft || configuration == Configurations::TopRight { 1 } else { 0 }
        } else {
            2 * clamped.y as usize | clamped.x as usize
        }
    }
}


