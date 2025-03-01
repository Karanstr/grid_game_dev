use std::cmp::{Reverse, Ordering};
use std::collections::{HashMap, BinaryHeap};
use macroquad::color::*;
use crate::globals::*;
use macroquad::math::{Vec2, BVec2, IVec2};
use crate::engine::grid::{partition::*, dag::{Index, ExternalPointer}};
use crate::engine::math::*;
use crate::engine::entities::{Location, ID, Entity};
use std::f32::consts::PI;

#[derive(Debug, Clone, derive_new::new)]
pub struct CollisionObject {
    pub target_location : Location,
    pub target_angular : f32,
    pub target : ID,
    pub owner_position : Vec2,
    pub owner_angular : f32,
    pub owner : ID,
    pub linear_velocity : Vec2,
    pub particles : BinaryHeap<Reverse<Particle>>,
}
impl CollisionObject {
    pub fn projected_owner(&self, ticks_into_projection: f32) -> Vec2 {
        (self.owner_position + self.linear_velocity*ticks_into_projection - self.target_location.position).rotate(Vec2::from_angle(self.target_angular * ticks_into_projection)) + self.target_location.position
    }
    pub fn instant_tangential_velocity(&self, offset: Vec2, ticks_into_projection: f32) -> Vec2 {
        self.linear_velocity
            + angular_to_tangential_velocity(self.owner_angular, offset)
            + angular_to_tangential_velocity(
                -self.target_angular,
                offset + self.projected_owner(ticks_into_projection) - self.target_location.position
            )
    }
}

#[derive(Debug, Clone, derive_new::new)]
pub struct Particle {
    pub offset : Vec2,
    pub corner_type : CornerType,
    #[new(value = "0.")]
    pub ticks_into_projection : f32,
}
impl PartialOrd for Particle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
impl Ord for Particle {
    fn cmp(&self, other: &Self) -> Ordering { 
        if self.ticks_into_projection.approx_eq(other.ticks_into_projection) { Ordering::Equal }
        else if self.ticks_into_projection.less(other.ticks_into_projection) { Ordering::Less }
        else if self.ticks_into_projection.greater(other.ticks_into_projection) { Ordering::Greater }
        else { unreachable!() }
    } 
}
impl PartialEq for Particle {
    fn eq(&self, other: &Self) -> bool { self.ticks_into_projection.approx_eq(other.ticks_into_projection) }
}
impl Eq for Particle {} 

pub enum CheckZorders {
    One(usize),
    Two([usize; 2]),
}

#[derive(Debug, Clone, Copy)]
pub enum CornerType {
    TopLeft,
    Top(f32),
    TopRight,
    Right(f32),
    BottomRight,
    Bottom(f32),
    BottomLeft,
    Left(f32),
}
impl CornerType {
    pub fn checks(&self, velocity:Vec2) -> CheckZorders {
        if velocity.is_zero() { panic!("AHHH (Velocity shouldn't be zero)"); }
        let (x, y) = (velocity.x.greater(0.) as usize, velocity.y.greater(0.) as usize);
        if velocity.x.is_zero() {
            match self {
                Self::Top(_) | Self::Bottom(_) => CheckZorders::Two([2 * y, (2 * y) | 1]),
                Self::TopLeft | Self::BottomLeft | Self::Left(_) => CheckZorders::One((2 * y) | 1),
                Self::TopRight | Self::BottomRight | Self::Right(_) => CheckZorders::One(2 * y),
            }
        } else if velocity.y.is_zero() {
            match self {
                Self::Left(_) | Self::Right(_) => CheckZorders::Two([2 | x, x]),
                Self::TopLeft | Self::TopRight | Self::Top(_) => CheckZorders::One(2 | x),
                Self::BottomLeft | Self::BottomRight | Self::Bottom(_) => CheckZorders::One(x),
            }
        } else { CheckZorders::One((2 * y) | x) }
    }
    pub fn hittable_walls(&self, velocity:Vec2) -> BVec2 {
        BVec2::from_array(match self {
            Self::TopLeft => [velocity.x.less(0.), velocity.y.less(0.)],
            Self::TopRight => [velocity.x.greater(0.), velocity.y.less(0.)],
            Self::BottomLeft => [velocity.x.less(0.), velocity.y.greater(0.)],
            Self::BottomRight => [velocity.x.greater(0.), velocity.y.greater(0.)],
            Self::Top(_) => [false, velocity.y.less(0.)],
            Self::Bottom(_) => [false, velocity.y.greater(0.)],
            Self::Left(_) => [velocity.x.less(0.), false],
            Self::Right(_) => [velocity.x.greater(0.), false],
        })
    }
    
    pub fn from_index(index: usize) -> Self {
        match index {
            0 => Self::TopLeft,
            1 => Self::TopRight,
            2 => Self::BottomLeft,
            3 => Self::BottomRight,
            _ => unimplemented!("Not sure how to do that.."),
        }
    }

    pub fn from_rotation(rotation: f32) -> Self {
        match rotation.rem_euclid(PI * 2.) {
            rot if rot.approx_eq(PI / 4.) => Self::BottomRight,
            rot if rot.approx_eq(PI * 3./4.) => Self::BottomLeft,
            rot if rot.approx_eq(PI * 5./4.) => Self::TopLeft,
            rot if rot.approx_eq(PI * 7./4.) => Self::TopRight,
            rot if rot.less(PI / 4.) => Self::Right(rot),
            rot if rot.less(PI * 3./4.) => Self::Bottom(rot),
            rot if rot.less(PI * 5./4.) => Self::Left(rot),
            rot if rot.less(PI * 7./4.) => Self::Top(rot),
            rot => Self::Right(rot),
        }
    }
    pub fn rotation(&self) -> f32 {
        match self {
            Self::BottomRight => PI/4.,
            Self::BottomLeft => PI * 3./4.,
            Self::TopLeft => PI * 5./4.,
            Self::TopRight => PI * 7./4.,
            Self::Top(angle) 
            | Self::Left(angle) 
            | Self::Right(angle) 
            | Self::Bottom(angle) => *angle,
        }
    }
    pub fn rotate(&self, rotation: f32) -> Self { Self::from_rotation(self.rotation() + rotation) }
}

#[derive(Debug)]
struct Hit {
    pub owner : ID,
    pub target : ID,
    pub walls : BVec2,
}

// Eventually turn this into an island generator
fn collect_collision_objects() -> Vec<CollisionObject> {
    let mut objects = Vec::new();
    let entities = ENTITIES.read();
    for idx in 0..entities.entities.len() {
        let owner = &entities.entities[idx];
        for other_idx in idx + 1..entities.entities.len() {
            let target = &entities.entities[other_idx];
            if let Some(obj) = entity_to_collision_object(owner, target) { 
                objects.push(obj); 
            }
            if let Some(obj) = entity_to_collision_object(target, owner) { 
                objects.push(obj); 
            }
        }
    }
    objects
}

fn apply_drag() {
    const DRAG_MULTIPLIER: f32 = 0.95;
    for entity in &mut ENTITIES.write().entities { 
        entity.velocity = (entity.velocity * DRAG_MULTIPLIER).snap_zero();
        entity.angular_velocity = (entity.angular_velocity * DRAG_MULTIPLIER).snap_zero();
    }
}

fn tick_entities(delta_tick: f32) {
    for entity in &mut ENTITIES.write().entities {
        entity.location.position += (entity.velocity * delta_tick).snap_zero();
        entity.rel_rotate((entity.angular_velocity * delta_tick).snap_zero());
    }
}

pub fn n_body_collisions(static_thing: ID) {
    let mut tick_max = 1.;
    loop {
        let objects = collect_collision_objects();
        let (actions, ticks_at_hit) = find_next_action(objects, tick_max);
        
        if actions.is_empty() {
            tick_entities(tick_max);
            break;
        } else {
            tick_max -= ticks_at_hit;
            tick_entities(ticks_at_hit);
        }
        
        let mut combined_hits = HashMap::new();
        for hit in &actions {
            *combined_hits.entry((hit.owner, hit.target)).or_insert(hit.walls) |= hit.walls;
        }
        let mut entities = ENTITIES.write();
        for ((owner_id, target_id), walls) in &combined_hits {
            let target = entities.get_entity(*target_id).unwrap();
            let rel_velocity = entities.get_entity(*owner_id).unwrap().velocity - target.velocity;
            let world_impulse = (-rel_velocity.rotate(Vec2::from_angle(-target.rotation)) * walls.as_vec2()).rotate(target.forward);
            let changes = [(owner_id, 1.), (target_id, -1.)];
            for (entity_id, multiplier) in changes {
                if *entity_id != static_thing {
                    let entity = entities.get_mut_entity(*entity_id).unwrap();
                    entity.velocity = (entity.velocity + world_impulse * multiplier).snap_zero();
                    entity.angular_velocity = 0.;
                }
            }
        }
    }
    apply_drag();
}

pub fn _just_move() {
    tick_entities(1.);
    apply_drag();
}

// Eventually extract this detection logic into our collision detection system (once we write it)
use super::raymarching::{Motion, Line};
fn find_next_action(objects:Vec<CollisionObject>, tick_max:f32) -> (Vec<Hit>, f32) {
    let mut ticks_to_action = tick_max;
    let mut action:Vec<Hit> = Vec::new();
    'objectloop : for mut object in objects {
        while let Some(Reverse(mut cur_corner)) = object.particles.pop() {
            if cur_corner.ticks_into_projection.greater(ticks_to_action) { continue 'objectloop }
            let motion = Motion::new(
                object.target_location.position,
                object.projected_owner(cur_corner.ticks_into_projection),
                cur_corner.offset,
                object.linear_velocity,
                object.target_angular,
                object.owner_angular,
            );
            // Why aren't we just passing object?
            let Some(ticks_to_hit) = next_intersection(
                motion,
                object.instant_tangential_velocity(cur_corner.offset, cur_corner.ticks_into_projection),
                object.target_location,
                cur_corner.corner_type,
                ticks_to_action,
            ) else { continue };
            cur_corner.ticks_into_projection += ticks_to_hit;
            cur_corner.offset = motion.project_to(ticks_to_hit) - object.projected_owner(cur_corner.ticks_into_projection);
            cur_corner.corner_type = cur_corner.corner_type.rotate(ticks_to_hit * (object.owner_angular - object.target_angular));
            if let Some(walls_hit) = hitting_wall(
                gate::point_to_real_cells(object.target_location, motion.project_to(ticks_to_hit)),
                object.instant_tangential_velocity(cur_corner.offset, cur_corner.ticks_into_projection),
                cur_corner.corner_type
            ) {
                if cur_corner.ticks_into_projection.less(ticks_to_action) { action.clear() }
                action.push( Hit {
                    owner : object.owner,
                    target : object.target,
                    walls : walls_hit,
                } );
                ticks_to_action = cur_corner.ticks_into_projection;
            } else { object.particles.push(Reverse(cur_corner)) }
        }
    }
    (action, ticks_to_action)
}

fn next_intersection(
    motion: Motion,
    itvel: Vec2,
    hitting_location: Location,
    corner_type: CornerType,
    tick_max: f32,
) -> Option<f32> {
    let point = motion.project_to(0.);
    CAMERA.read().draw_point(point, 0.02, RED);
    let radius = center_to_edge(hitting_location.pointer.height, hitting_location.min_cell_length);

    let cells = gate::point_to_real_cells(hitting_location, point);
    if hitting_wall(cells, itvel, corner_type).is_some() { return Some(0.) }
    let index = 2 * (itvel.y.greater(0.) as usize) | (itvel.x.greater(0.) as usize);
    let grid_top_left = hitting_location.position - radius;
    let (top_left, bottom_right) = if let Some(cell) = cells[index] {
        let cell_length = cell_length(cell.pointer.height, hitting_location.min_cell_length);
        (grid_top_left + cell.cell.as_vec2() * cell_length, grid_top_left + (cell.cell + 1).as_vec2() * cell_length)
    } else { ( grid_top_left, hitting_location.position + radius) };

    let mut ticks_to_hit = f32::INFINITY;
    for bound in [top_left, bottom_right] {
        for i in 0 .. 2 {
            if point[i].approx_eq(bound[i]) { continue };
            let line = if i == 0 { Line::Vertical(bound[i]) } else { Line::Horizontal(bound[i]) };
            let Some(tick) = motion.solve_all(line, tick_max.min(ticks_to_hit)) else { continue };
            ticks_to_hit = ticks_to_hit.min(tick);
        }
    }
    (ticks_to_hit.less_eq(tick_max)).then_some(ticks_to_hit)
}

pub fn entity_to_collision_object(owner:&Entity, target:&Entity) -> Option<CollisionObject> {
    let mut collision_points = BinaryHeap::new();
    let offset = center_to_edge(owner.location.pointer.height, owner.location.min_cell_length);
    let align_target = Vec2::from_angle(-target.rotation);
    let rel_velocity = (owner.velocity - target.velocity).rotate(align_target).snap_zero();
    if rel_velocity.is_zero() && (owner.angular_velocity - target.angular_velocity).is_zero() { return None }
    let rotated_owner_pos = (owner.location.position - target.location.position).rotate(align_target) + target.location.position;
    for corners in owner.corners.iter() {
        for i in 0..4 {
            // Cull any corner which isn't exposed
            if corners.mask & (1 << i) == 0 { continue }
            let offset = ((corners.points[i] - offset).rotate(owner.forward) + owner.location.position - target.location.position)
                .rotate(align_target) + target.location.position - rotated_owner_pos;
            collision_points.push(Reverse(Particle::new(
                offset,
                CornerType::from_index(i).rotate(owner.rotation - target.rotation)
            )));
        }
    }
    Some(CollisionObject::new(
        target.location,
        target.angular_velocity,
        target.id,
        rotated_owner_pos,
        owner.angular_velocity,
        owner.id,
        rel_velocity,
        collision_points
    ))
}

// Extract corners into our collision detection system (once we write it)
#[derive(Debug, Clone, derive_new::new)]
pub struct Corners {
    pub points : [Vec2; 4],
    pub index : Index,
    pub mask : u8,
}

pub mod corner_handling {
    use super::*;

    fn cell_corner_mask(start: ExternalPointer, zorder: ZorderPath) -> u8 {
        const CORNER_CHECKS: [([(IVec2, u8); 3], u8); 4] = [
            // Format: ([(offset, step_direction), ...], corner_mask_bit)
            ([(IVec2::new(-1, 0), 0b01), (IVec2::new(0, -1), 0b10), (IVec2::new(-1, -1), 0b11)], 0b0001), // Top Left
            ([(IVec2::new(1, 0), 0b00), (IVec2::new(0, -1), 0b11), (IVec2::new(1, -1), 0b10)], 0b0010),  // Top Right
            ([(IVec2::new(-1, 0), 0b11), (IVec2::new(0, 1), 0b00), (IVec2::new(-1, 1), 0b01)], 0b0100),  // Bottom Left
            ([(IVec2::new(1, 0), 0b10), (IVec2::new(0, 1), 0b01), (IVec2::new(1, 1), 0b00)], 0b1000),    // Bottom Right
        ];

        let mut exposed_mask = 0b0000;
        'corner: for (checks, mask) in CORNER_CHECKS {
            for (offset, direction) in checks {
                let Some(mut check_zorder) = zorder.move_cartesianly(offset) else { continue };
                for _ in 0 .. start.height - check_zorder.depth {
                    check_zorder = check_zorder.step_down(direction as u32)
                }
                let pointer = GRAPH.read().read(start, &check_zorder.steps()).unwrap();
                if BLOCKS.is_solid_index(*pointer.pointer) { continue 'corner }
            }
            exposed_mask |= mask;
        }
        exposed_mask
    }

    //The top left corner of the root is (0, 0)
    fn cell_corners(cell:CellData, min_cell_length:Vec2) -> [Vec2; 4] {
        let cell_size = cell_length(cell.pointer.height, min_cell_length);
        let top_left_corner = cell.cell.as_vec2() * cell_size;
        [
            top_left_corner,
            top_left_corner.with_x(top_left_corner.x + cell_size.x),
            top_left_corner.with_y(top_left_corner.y + cell_size.y),
            top_left_corner + cell_size,
        ]
    }

    pub fn tree_corners(start:ExternalPointer, min_cell_length:Vec2) -> Vec<Corners> {
        let leaves = GRAPH.read().dfs_leaf_cells(start);
        let mut corners = Vec::new();
        for cell in leaves {
            let zorder = ZorderPath::from_cell(cell.cell, start.height - cell.pointer.height);
            corners.push( Corners::new(
                cell_corners(cell, min_cell_length),
                cell.pointer.pointer,
                if !BLOCKS.is_solid_index(*cell.pointer.pointer) { 0 } else { cell_corner_mask(start, zorder) }
            ));
        }
        corners 
    }
    
    
}

fn hitting_wall(position_data:[Option<CellData>; 4], itvel:Vec2, corner_type:CornerType) -> Option<BVec2> {
    let mut hit_walls = corner_type.hittable_walls(itvel);
    // If we're inside air cells, we should override the wall check.
    hit_walls &= BVec2::splat(match corner_type.checks(itvel) {
        CheckZorders::One(idx) => BLOCKS.is_solid_cell(position_data[idx]),
        CheckZorders::Two([idx1, idx2]) => BLOCKS.is_solid_cell(position_data[idx1]) | BLOCKS.is_solid_cell(position_data[idx2]),
    });
    // If we're on a wall, check if we should be sliding or stopping.
    if hit_walls == BVec2::TRUE {
        let idxs = match itvel.signum() {
            Vec2{x: -1., y: -1.} => [2, 1],
            Vec2{x: -1., y: 1.} => [0, 3],
            Vec2{x: 1., y: -1.} => [3, 0],
            Vec2{x: 1., y: 1.} => [1, 2],
            _ => unreachable!(),
        };
        let slide = BVec2::new(
            BLOCKS.is_solid_cell(position_data[idxs[0]]),
            BLOCKS.is_solid_cell(position_data[idxs[1]])
        );
        // If we're on a corner (both air), we should still stop
        if slide != BVec2::FALSE { hit_walls &= slide }
    };
    // None represents no walls nicer than BVec::FALSE does imo
    (hit_walls != BVec2::FALSE).then_some(hit_walls)
}
