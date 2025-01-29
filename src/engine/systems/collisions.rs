use std::cmp::{Reverse, Ordering};
use std::collections::BinaryHeap;
use super::*;

const EPSILON: f32 = 1e-6;

fn approx_eq(a: f32, b: f32) -> bool {
    (a - b).abs() < EPSILON
}

fn vec2_approx_eq(a: Vec2, b: Vec2) -> bool {
    approx_eq(a.x, b.x) && approx_eq(a.y, b.y)
}

fn vec2_remove_err(a: Vec2) -> Vec2 {
    Vec2::new(if a.x.abs() < EPSILON { 0. } else {a.x}, if a.y.abs() < EPSILON { 0. } else {a.y})
}

#[derive(Debug, Clone, new)]
pub struct CollisionObject {
    pub position : Vec2, //Grid Center
    pub velocity : Vec2,
    // pub angular_velocity : f32,
    // pub CoR : Vec2, //Center of Rotation
    pub owner : ID,
    pub hitting : ID,
    pub particles : BinaryHeap<Reverse<Particle>>,
}

#[derive(Debug, Clone, new)]
pub struct Particle {
    pub offset : Vec2,
    pub rotation : f32,
    #[new(value = "0.")]
    pub ticks_into_projection : f32,
    #[new(value = "[None; 4]")]
    pub position_data : [Option<CellData>; 4],
    pub type_of : usize,
}

#[derive(Debug, Clone, PartialEq)]
enum CellType {
    Solid,  // index 1 or 3
    Air,    // index 0 or 2
    Void,   // None
}

// Get the relevant Z-order indices based on velocity direction
// Returns (indices, is_vertical)
// For single-axis movement: returns 2 indices
// For diagonal movement: returns 1 index
fn get_relevant_indices(velocity:Vec2) -> (Vec<usize>, bool) {
    if velocity.x == 0. {
        // Moving vertically
        let indices = if velocity.y < 0. {
            vec![0, 1] // Moving up: check top cells (0,1)
        } else {
            vec![2, 3] // Moving down: check bottom cells (2,3)
        };
        (indices, true)
    } else if velocity.y == 0. {
        // Moving horizontally
        let indices = if velocity.x < 0. {
            vec![0, 2] // Moving left: check left cells (0,2)
        } else {
            vec![1, 3] // Moving right: check right cells (1,3)
        };
        (indices, false)
    } else {
        // Moving diagonally - only need to check one corner based on direction
        let idx = match (velocity.x > 0., velocity.y > 0.) {
            (false, false) => 0, // Moving top-left: check top-left corner
            (true, false) => 1,  // Moving top-right: check top-right corner
            (false, true) => 2,  // Moving bottom-left: check bottom-left corner
            (true, true) => 3,   // Moving bottom-right: check bottom-right corner
        };
        (vec![idx], false)
    }
}

impl PartialOrd for Particle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.ticks_into_projection.partial_cmp(&other.ticks_into_projection)
    }
}
impl Ord for Particle {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}
impl PartialEq for Particle {
    fn eq(&self, other: &Self) -> bool {
        self.ticks_into_projection == other.ticks_into_projection
    }
}
impl Eq for Particle {} 
impl Particle {
    fn position(&self, owner:&CollisionObject) -> Vec2 {
        self.offset + owner.position + owner.velocity * self.ticks_into_projection
    }

}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WallSign {
    None,
    Negative,
    Positive
}
#[derive(Debug, Clone, Copy, PartialEq, new)]
pub struct WallTouch {
    pub horizontal: WallSign,
    pub vertical: WallSign,
}

impl WallTouch {
    // Given a single-axis velocity, returns the index in the direction of the wall
    pub fn to_zorder_index(&self, velocity: Vec2) -> Option<usize> {
        if velocity.x != 0.0 && velocity.y != 0.0 { dbg!("AHHH"); return None }
        if velocity.x == 0.0 && velocity.y == 0.0 { dbg!("AHHH2"); return None }
        let clamped = velocity.signum().max(Vec2::ZERO);
        Some( if velocity.x == 0. {
            2 * clamped.y as usize | if self.horizontal == WallSign::Positive { 0 } else { 1 } 
        } else {
            clamped.x as usize | 2 * if self.vertical == WallSign::Positive { 0 } else { 1 } 
        } )
        
    }
}

struct Hit {
    pub owner : ID,
    pub hitting : ID,
    pub walls : BVec2,
    pub ticks : f32,
}

// Determines which specific walls of the bounding box a corner touches given its rotation in radians
// Needs to know which corner we're in
pub fn corner_wall_collision(corner: usize) -> WallTouch {
    match corner {
        0 => WallTouch::new(WallSign::Negative, WallSign::Negative),
        1 => WallTouch::new(WallSign::Positive, WallSign::Negative),
        2 => WallTouch::new(WallSign::Negative, WallSign::Positive),
        3 => WallTouch::new(WallSign::Positive, WallSign::Positive),
        _ => unreachable!(),
    }
}

pub fn n_body_collisions(entities:&mut EntityPool, static_thing:ID) {
    let mut tick_max = 1.;
    loop {
        let mut objects = Vec::new();
        for idx in 0 .. entities.entities.len() {
            let entity = &entities.entities[idx];
            for other_idx in idx + 1 .. entities.entities.len() {
                let other = &entities.entities[other_idx];
                if within_range(&entity, &other) {
                    if let Some(obj) = entity_to_collision_object(&entity, &other) { objects.push(obj); }
                    if let Some(obj) = entity_to_collision_object(&other, &entity) { objects.push(obj); }
                }
            }
        }
        
        let Some(hit) = find_next_action(&entities, objects.clone(), tick_max) else {
            for entity in &mut entities.entities {
                let delta = entity.velocity * tick_max;
                // Skip tiny movements that could cause precision issues
                if !vec2_approx_eq(delta, Vec2::ZERO) { entity.location.position += delta;}
            }
            break
        };
        // Update positions with error checking
        for entity in &mut entities.entities {
            let delta = entity.velocity * hit.ticks;
            // Skip tiny movements that could cause precision issues
            if !vec2_approx_eq(delta, Vec2::ZERO) { entity.location.position += delta;}
        }
        tick_max -= hit.ticks;
        
        let hitting = entities.get_entity(hit.hitting).unwrap();
        let world_to_hitting = Mat2::from_angle(-hitting.rotation);
        
        let walls_as_int = IVec2::from(hit.walls).as_vec2();
        let relative_velocity = world_to_hitting.mul_vec2(
            entities.get_entity(hit.owner).unwrap().velocity - hitting.velocity
        );
        
        let impulse = vec2_remove_err(world_to_hitting.transpose().mul_vec2(-relative_velocity * walls_as_int));
        if hit.owner != static_thing {
            let entity = entities.get_mut_entity(hit.owner).unwrap();
            entity.velocity += impulse;
            entity.velocity = vec2_remove_err(entity.velocity);
        }
        if hit.hitting != static_thing {
            let entity = entities.get_mut_entity(hit.hitting).unwrap();
            entity.velocity -= impulse;
            entity.velocity = vec2_remove_err(entity.velocity);
        }
    }
    let drag_multiplier = 0.95;
    for entity in &mut entities.entities { 
        entity.velocity *= drag_multiplier;
        entity.velocity = vec2_remove_err(entity.velocity);
    }

}

// Eventually make this work with islands, solving each island by itself
fn find_next_action(entities:&EntityPool, objects:Vec<CollisionObject>, tick_max:f32) -> Option<Hit> {
    let mut ticks_to_action = tick_max;
    let mut action = None;
    'objectloop : for mut object in objects {
        while let Some(Reverse(mut cur_corner)) = object.particles.pop() {
            if cur_corner.ticks_into_projection >= ticks_to_action { continue 'objectloop }
            let hitting_location = entities.get_entity(object.hitting).unwrap().location;
            let Some(ticks_to_hit) = next_intersection(
                cur_corner.position(&object),
                object.velocity,
                cur_corner.rotation, //Remember to recompute for ticks into projection
                cur_corner.position_data,
                hitting_location,
                tick_max,
                cur_corner.type_of
            ) else { continue };
            cur_corner.ticks_into_projection += ticks_to_hit;
            let position_data = gate::point_to_real_cells(
                hitting_location,
                cur_corner.position(&object)
            );
            cur_corner.position_data = position_data;
            
            if let Some(walls_hit) = hitting_wall(cur_corner.position_data, object.velocity, cur_corner.rotation, cur_corner.type_of) {
                // dbg!(walls_hit);
                // if walls_hit.y { dbg!(&cur_corner); }
                action = Some( Hit {
                        owner : object.owner,
                        hitting : object.hitting,
                        walls : walls_hit,
                        ticks : cur_corner.ticks_into_projection
                    }
                );
                ticks_to_action = cur_corner.ticks_into_projection;
            } else { object.particles.push(Reverse(cur_corner)) }
        }
    }
    action
}

// Rename this
// Checks if a point is at a boundary and determines the cell offset based on velocity
fn check_boundary_collision(point: Vec2, velocity: Vec2, top_left: Vec2, bottom_right: Vec2) -> Option<Vec2> {
    let mut cell = Vec2::ZERO;
    
    // Check x-axis boundaries
    if point.x <= top_left.x + EPSILON {
        if velocity.x > 0. { cell.x = -1. } else { return None }
    } else if point.x >= bottom_right.x - EPSILON {
        if velocity.x < 0. { cell.x = 1. } else { return None }
    }
    
    // Check y-axis boundaries
    if point.y <= top_left.y + EPSILON {
        if velocity.y > 0. { cell.y = -1. } else { return None }
    } else if point.y >= bottom_right.y - EPSILON {
        if velocity.y < 0. { cell.y = 1. } else { return None }
    }
    
    Some(cell)
}

// Selects the appropriate cell and height based on position data and indices
fn select_cell_and_height(
    position_data: &[Option<CellData>; 4],
    rel_idxs: &[usize],
    point: Vec2,
    velocity: Vec2,
    top_left: Vec2,
    bottom_right: Vec2,
    default_height: u32,
) -> Option<(Vec2, u32)> {
    // Helper function to get cell data from an index
    let get_cell_data = |idx: usize| {
        position_data[idx].as_ref().map(|data| (data.cell.as_vec2(), data.pointer.height))
    };

    // Helper function for boundary collision
    let check_boundary = || {
        check_boundary_collision(point, velocity, top_left, bottom_right)
            .map(|cell| (cell, default_height))
    };

    match rel_idxs.len() {
        1 => get_cell_data(rel_idxs[0]).or_else(check_boundary),
        2 => {
            let data0 = get_cell_data(rel_idxs[0]);
            let data1 = get_cell_data(rel_idxs[1]);
            
            match (data0, data1) {
                (None, None) => check_boundary(),
                (None, Some(data)) | (Some(data), None) => Some(data),
                (Some((cell0, height0)), Some((cell1, height1))) => {
                    Some( if height0 < height1 { (cell0, height0) } else { (cell1, height1) } )
                }
            }
        }
        _ => None,
    }
}

fn next_intersection(
    point: Vec2,
    velocity: Vec2,
    rotation: f32,
    position_data: [Option<CellData>; 4],
    hitting_location: Location,
    tick_max: f32,
    corner_type: usize,
) -> Option<f32> {
    let hitting_aabb = bounds::aabb(hitting_location.position, hitting_location.pointer.height);
    let top_left = hitting_aabb.min();
    let bottom_right = hitting_aabb.max();
    let within_bounds = hitting_aabb.contains(point);

    // Consider flipping velocity and returning ticks back to wall.
    if hitting_wall(position_data, velocity, rotation, corner_type).is_some() { return Some(0.); }

    let (rel_idxs, _) = get_relevant_indices(velocity);
    
    let (cell, height) = select_cell_and_height(
        &position_data,
        &rel_idxs,
        point,
        velocity,
        top_left,
        bottom_right,
        hitting_location.pointer.height,
    )?;

    let quadrant = velocity.signum().max(Vec2::ZERO);
    let cell_length = bounds::cell_length(height);
    let boundary_corner = top_left + cell * cell_length + cell_length * quadrant;
    let ticks = (boundary_corner - point) / velocity;

    let ticks_to_hit = match (within_bounds.x, within_bounds.y) {
        (false, false) => ticks.max_element(),
        (true, false) if ticks.x == 0. => ticks.y,
        (false, true) if ticks.y == 0. => ticks.x,
        _ => ticks.min_element(),
    };

    if ticks_to_hit.is_nan() || ticks_to_hit.abs() > tick_max + EPSILON {
        None
    } else {
        Some(ticks_to_hit)
    }
}

//Make this work again
pub fn within_range(entity1:&Entity, entity2:&Entity) -> bool {
    let aabb = bounds::aabb(entity1.location.position, entity1.location.pointer.height).expand(entity1.velocity);
    let aabb2 = bounds::aabb(entity2.location.position, entity2.location.pointer.height).expand(entity2.velocity);
    let result = aabb.intersects(aabb2) == BVec2::TRUE;
    let color = if result { GREEN } else { RED };
    let camera = CAMERA.read().unwrap();
    camera.outline_bounds(aabb, 0.05, color);
    camera.outline_bounds(aabb2, 0.05, color);
    true //result
}

// Add culling edgecase for no rotation
//Relative angles only relevant when culling
//We aren't culling rn
pub fn entity_to_collision_object(owner:&Entity, hitting:&Entity) -> Option<CollisionObject> {
    let mut collision_points = BinaryHeap::new();
    let align_to_hitting = Vec2::from_angle(-hitting.rotation);
    let offset = bounds::center_to_edge(owner.location.pointer.height);
    //Worldspace to hitting aligned
    let rel_velocity = vec2_remove_err((owner.velocity - hitting.velocity).rotate(align_to_hitting));
    if rel_velocity.length() < EPSILON { return None }
    let rotated_owner_pos = (owner.location.position - hitting.location.position).rotate(align_to_hitting) + hitting.location.position;
    let camera = CAMERA.read().unwrap();
    camera.draw_point(rotated_owner_pos, 0.1, GREEN);
    for corners in owner.corners.iter() {
        for i in 0..4 {
            //Cull any corner which isn't a corner
            if corners.mask & (1 << i) == 0 { continue }
            let point = (corners.points[i] - offset).rotate(align_to_hitting).rotate(owner.forward);
            camera.draw_point(point + rotated_owner_pos, 0.1, RED);
            let mut particle = Particle::new(point, -hitting.rotation + owner.rotation, i);
            particle.position_data = gate::point_to_real_cells(hitting.location, point + rotated_owner_pos);
            collision_points.push(Reverse(particle));
        }
    }
    Some(CollisionObject::new(
        rotated_owner_pos,
        rel_velocity,
        owner.id,
        hitting.id,
        collision_points
    ))
}


#[derive(Debug, Clone, new)]
pub struct Corners {
    pub points : [Vec2; 4],
    pub index : Index,
    pub mask : u8,
}

pub mod corner_handling {
    use super::*;

    //Figure out if this can be improved?
    fn cell_corner_mask(start:ExternalPointer, zorder:ZorderPath) -> u8 {
            let mut exposed_mask = 0b1111;
            let checks = [
                (IVec2::new(-1, 0), 0b01), //Top Left 0
                (IVec2::new(0, -1), 0b10),
                (IVec2::new(-1, -1), 0b11),
                (IVec2::new(1, 0), 0b00), //Top Right 1
                (IVec2::new(0, -1), 0b11),
                (IVec2::new(1, -1), 0b10),
                (IVec2::new(-1, 0), 0b11), //Bottom Left 2
                (IVec2::new(0, 1), 0b00),
                (IVec2::new(-1, 1), 0b01),
                (IVec2::new(1, 0), 0b10), //Bottom Right 3
                (IVec2::new(0, 1), 0b01),
                (IVec2::new(1, 1), 0b00),
            ];
            for i in 0 .. 4 {
                for j in 0 .. 3 {
                    let (offset, direction) = checks[i*3 + j];
                    let Some(mut check_zorder) = zorder.move_cartesianly(offset) else { continue };
                    for _ in 0 .. start.height - check_zorder.depth {
                        check_zorder = check_zorder.step_down(direction)
                    }
                    let pointer = GRAPH.read().unwrap().read(start, &check_zorder.steps()).unwrap();
                    //Add proper block lookup
                    if !is_ignore(pointer.pointer) { exposed_mask -= 1 << i; break }
                }
            }
            exposed_mask
        }

    //The top left corner of the root is (0, 0)
    fn cell_corners(cell:CellData) -> [Vec2; 4] {
        let cell_size = bounds::cell_length(cell.pointer.height);
        let top_left_corner = cell.cell.as_vec2() * cell_size;
        [
            top_left_corner,
            top_left_corner.with_x(top_left_corner.x + cell_size.x),
            top_left_corner.with_y(top_left_corner.y + cell_size.y),
            top_left_corner + cell_size,
        ]
    }

    pub fn tree_corners(start:ExternalPointer) -> Vec<Corners> {
        let leaves = GRAPH.read().unwrap().dfs_leaf_cells(start);
        let mut corners = Vec::new();
        for cell in leaves {
            let zorder = ZorderPath::from_cell(cell.cell, start.height - cell.pointer.height);
            corners.push( Corners::new(
                cell_corners(cell),
                cell.pointer.pointer,
                if is_ignore(cell.pointer.pointer) { 0 } else { cell_corner_mask(start, zorder) }
            ));
        }
        corners 
    }
    
    
}

fn slide_check(velocity:Vec2, position_data:[Option<CellData>; 4]) -> BVec2 {
    let (x_slide_check, y_slide_check) = if velocity.x < 0. && velocity.y < 0. { //(-,-)
        (2, 1)
    } else if velocity.x < 0. && velocity.y > 0. { //(-,+)
        (0, 3)
    } else if velocity.x > 0. && velocity.y < 0. { //(+,-)
        (3, 0)
    } else { //(+,+)
        (1, 2)
    };
    let result = BVec2::new(
        is_solid(&position_data, x_slide_check),
        is_solid(&position_data, y_slide_check),
    );
    result
}

fn check_wall(position_data:&[Option<CellData>; 4], idx: usize) -> CellType {
    match &position_data[idx] {
        Some(cell_data) => {
            let index = *cell_data.pointer.pointer;
            if index == 1 || index == 3 { CellType::Solid } else { CellType::Air }
        }
        None => CellType::Void
    }
}

fn hitting_wall(position_data:[Option<CellData>; 4], velocity:Vec2, rotation: f32, corner_type:usize) -> Option<BVec2> {
    if velocity == Vec2::ZERO { return None }

    let (indices, is_vertical) = get_relevant_indices(velocity);
    let result = if velocity.x == 0. || velocity.y == 0. {
        // Single-axis movement
        let collisions = corner_wall_collision(corner_type);
        let wall = is_solid(&position_data, collisions.to_zorder_index(velocity).unwrap_or(0)); 
        // let wall = indices.iter().any(|&idx| is_solid(&position_data, idx));
        BVec2::new(wall & !is_vertical, wall & is_vertical)
    } else {
        // Diagonal movement - if the corner is solid, we're hitting in both directions
        BVec2::splat(is_solid(&position_data, indices[0]))
    };
    if result == BVec2::TRUE {
        let slide_checked = slide_check(velocity, position_data);
        if slide_checked == BVec2::FALSE { Some(BVec2::TRUE) } else { Some(slide_checked) }
    } else if result == BVec2::FALSE { None } else { 
        Some(result)
    }
}


//Figure out unifying these
fn is_solid(position_data:&[Option<CellData>; 4], idx: usize) -> bool {
    matches!(check_wall(&position_data, idx), CellType::Solid)
}
//Change this name
fn is_ignore(index:Index) -> bool { *index == 2 || *index == 0 }
