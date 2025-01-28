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

    fn real_position(&self, owner:&CollisionObject) -> Vec2 {
        self.position(owner)
    }

}

//Unify these later, negative, none, positive
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WallSide {
    None,
    Left,
    Right
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WallHeight {
    None,
    Top,
    Bottom
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WallTouch {
    pub horizontal: WallHeight,
    pub vertical: WallSide,
}

impl WallTouch {
    /// Converts the wall touch into a z-order index for position_data array
    /// Given a single-axis velocity, returns the index opposite to the touched wall
    /// e.g., if touching left wall, returns right wall index
    pub fn to_zorder_index(&self, velocity: Vec2) -> Option<usize> {
        if velocity.x != 0.0 && velocity.y != 0.0 {
            return None; // Only works for single-axis velocity
        }

        match (velocity.x != 0.0, &self.vertical, &self.horizontal) {
            // Horizontal movement
            (true, WallSide::Left, _) => Some(1),   // Right side index
            (true, WallSide::Right, _) => Some(3),  // Left side index
            // Vertical movement
            (false, _, WallHeight::Top) => Some(2),    // Bottom side index
            (false, _, WallHeight::Bottom) => Some(0), // Top side index
            _ => None,
        }
    }
}

struct Hit {
    pub owner : ID,
    pub hitting : ID,
    pub walls : BVec2,
    pub ticks : f32,
}

// Determines which specific walls of the bounding box a corner touches given its rotation in radians
pub fn corner_wall_collision(rotation: f32) -> WallTouch {
    use std::f32::consts::PI;
    
    // Normalize rotation to 0-2π radians
    let rotation = rotation % (2.0 * PI);
    let rotation = if rotation < 0.0 { rotation + 2.0 * PI } else { rotation };

    // At exact 90-degree intervals (π/2, 3π/2, etc), the corner touches two walls
    if (rotation % (PI/2.0)).abs() < f32::EPSILON {
        let quadrant = (rotation / (PI/2.0)).round() as i32 % 4;
        return match quadrant {
            0 => WallTouch { vertical: WallSide::Right, horizontal: WallHeight::None },
            1 => WallTouch { vertical: WallSide::None, horizontal: WallHeight::Bottom },
            2 => WallTouch { vertical: WallSide::Left, horizontal: WallHeight::None },
            _ => WallTouch { vertical: WallSide::None, horizontal: WallHeight::Top },
        };
    }

    // Check which quadrant we're in and determine wall touches
    let vertical = match rotation {
        r if r <= PI/4.0 || r > 7.0*PI/4.0 => WallSide::Right,
        r if r > 3.0*PI/4.0 && r <= 5.0*PI/4.0 => WallSide::Left,
        _ => WallSide::None,
    };

    let horizontal = match rotation {
        r if r > PI/4.0 && r <= 3.0*PI/4.0 => WallHeight::Bottom,
        r if r > 5.0*PI/4.0 && r <= 7.0*PI/4.0 => WallHeight::Top,
        _ => WallHeight::None,
    };

    WallTouch { horizontal, vertical }
}

pub fn n_body_collisions<T:GraphNode>(entities:&mut EntityPool, graph:&SparseDirectedGraph<T>, camera:&Camera, static_thing:ID) {
    let mut tick_max = 1.;
    loop {
        let mut objects = Vec::new();
        for idx in 0 .. entities.entities.len() {
            let entity = &entities.entities[idx];
            for other_idx in idx + 1 .. entities.entities.len() {
                let other = &entities.entities[other_idx];
                if within_range(&entity, &other, camera) {
                    // objects.push(entity_to_collision_object(graph, &entity, &other, &camera));
                    objects.push(entity_to_collision_object(graph, &other, &entity, &camera));
                }
            }
        }
        
        let Some(hit) = find_next_action(&entities, &graph, objects.clone(), tick_max, camera) else {
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

//Eventually make this work with islands, solving each island by itself
fn find_next_action<T:GraphNode>(entities:&EntityPool, graph:&SparseDirectedGraph<T>, objects:Vec<CollisionObject>, tick_max:f32, camera:&Camera) -> Option<Hit> {
    let mut ticks_to_action = tick_max;
    let mut action = None;
    'objectloop : for mut object in objects {
        while let Some(Reverse(mut cur_corner)) = object.particles.pop() {
            if cur_corner.ticks_into_projection >= ticks_to_action { continue 'objectloop }
            let hitting_location = entities.get_entity(object.hitting).unwrap().location;
            let Some(ticks_to_hit) = next_intersection(
                cur_corner.position(&object),
                object.velocity,
                cur_corner.position_data,
                hitting_location,
                ticks_to_action,
                camera
            ) else { continue };
            cur_corner.ticks_into_projection += ticks_to_hit;
            let position_data = gate::point_to_real_cells(
                graph,
                hitting_location,
                cur_corner.position(&object)
            );
            cur_corner.position_data = position_data;
            
            if let Some(walls_hit) = hitting_wall(cur_corner.position_data, object.velocity) {
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

// Clean this up later
//Allow the bounding shape to be specified as anything which can be represented as a series of line intersections?
//Replace with fancyintersection code
fn next_intersection(point:Vec2, velocity:Vec2, position_data:[Option<CellData>; 4], hitting_location:Location, tick_max:f32, camera:&Camera) -> Option<f32> {
    let hitting_aabb = bounds::aabb(hitting_location.position, hitting_location.pointer.height);
    let top_left = hitting_aabb.min();
    let bottom_right = hitting_aabb.max();
    let within_bounds = hitting_aabb.contains(point);
    if hitting_wall(position_data, velocity).is_some() { return Some(0.) }
    let (rel_idxs, _) = get_relevant_indices(velocity);
    //Wow this is gross.
    let (cell, height) = if rel_idxs.len() == 1 {
        if position_data[rel_idxs[0]].is_none() {
            let mut cell = Vec2::ZERO;
            if point.x <= top_left.x + EPSILON {
                if velocity.x > 0. { cell.x = -1. } else { return None }
            } else if point.x >= bottom_right.x - EPSILON {
                if velocity.x < 0. { cell.x = 1. } else { return None }
            }
            if point.y <= top_left.y + EPSILON {
                if velocity.y > 0. { cell.y = -1. } else { return None }
            } else if point.y >= bottom_right.y - EPSILON {
                if velocity.y < 0. { cell.y = 1. } else { return None }
            }
            (cell, hitting_location.pointer.height)
        } else { (position_data[rel_idxs[0]].unwrap().cell.as_vec2(), position_data[rel_idxs[0]].unwrap().pointer.height) }
    } else {
        if position_data[rel_idxs[0]].is_none() && position_data[rel_idxs[1]].is_none() {
            let mut cell = Vec2::ZERO;
            if point.x <= top_left.x + EPSILON {
                if velocity.x > 0. { cell.x = -1. } else { return None }
            } else if point.x >= bottom_right.x - EPSILON {
                if velocity.x < 0. { cell.x = 1. } else { return None }
            }
            if point.y <= top_left.y + EPSILON {
                if velocity.y > 0. { cell.y = -1. } else { return None }
            } else if point.y >= bottom_right.y - EPSILON {
                if velocity.y < 0. { cell.y = 1. } else { return None }
            }
            (cell, hitting_location.pointer.height)
        } else if position_data[rel_idxs[0]].is_none() {
            (position_data[rel_idxs[1]].unwrap().cell.as_vec2(), position_data[rel_idxs[1]].unwrap().pointer.height)
        } else if position_data[rel_idxs[1]].is_none() {
            (position_data[rel_idxs[0]].unwrap().cell.as_vec2(), position_data[rel_idxs[0]].unwrap().pointer.height)
        } else {
            if position_data[rel_idxs[0]].unwrap().pointer.height < position_data[rel_idxs[1]].unwrap().pointer.height {
                (position_data[rel_idxs[0]].unwrap().cell.as_vec2(), position_data[rel_idxs[0]].unwrap().pointer.height)
            } else {
                (position_data[rel_idxs[1]].unwrap().cell.as_vec2(), position_data[rel_idxs[1]].unwrap().pointer.height)
            }
        }
    };
    let quadrant = velocity.signum().max(Vec2::ZERO);
    let cell_length = bounds::cell_length(height);
    let boundary_corner = top_left + cell * cell_length + cell_length * quadrant;
    let ticks = (boundary_corner - point) / velocity; 
    let ticks_to_hit = match (within_bounds.x, within_bounds.y) {
        (false, false) => { ticks.max_element() },
        (true, false) if ticks.x == 0. => { ticks.y },
        (false, true) if ticks.y == 0. => { ticks.x },
        _ => { ticks.min_element() },
    };
    if ticks_to_hit.is_nan() || ticks_to_hit.abs() > tick_max + EPSILON { None }
    else { Some(ticks_to_hit) }
}

pub fn within_range(entity1:&Entity, entity2:&Entity, camera:&Camera) -> bool {
    let aabb = bounds::aabb(entity1.location.position, entity1.location.pointer.height).expand(entity1.velocity);
    let aabb2 =  bounds::aabb(entity2.location.position, entity2.location.pointer.height).expand(entity2.velocity);
    let result = aabb.intersects(aabb2) == BVec2::TRUE;
    let color = if result { RED } else { RED };
    camera.outline_bounds(aabb, 0.05, color);
    camera.outline_bounds(aabb2, 0.05, color);
    true // result
}

// Add culling edgecase for no rotation
//Relative angles only relevant when culling
//We aren't culling rn
pub fn entity_to_collision_object<T:GraphNode>(graph:&SparseDirectedGraph<T>, owner:&Entity, hitting:&Entity, camera:&Camera) -> CollisionObject {
    let mut collision_points = BinaryHeap::new();
    let align_to_hitting = Vec2::from_angle(-hitting.rotation);
    let offset = bounds::center_to_edge(owner.location.pointer.height);
    //Worldspace to hitting aligned
    let rel_velocity = vec2_remove_err((owner.velocity - hitting.velocity).rotate(align_to_hitting));
    let rotated_owner_pos = (owner.location.position - hitting.location.position).rotate(align_to_hitting) + hitting.location.position;
    camera.draw_point(rotated_owner_pos, 0.1, GREEN);
    for corners in owner.corners.iter() {
        for i in 0..4 {
            //Cull any corner which isn't a corner
            if corners.mask & (1 << i) == 0 { continue }
            let point = (corners.points[i] - offset).rotate(align_to_hitting).rotate(owner.forward);
            camera.draw_point(point + rotated_owner_pos, 0.1, RED);
            let mut particle = Particle::new(point, -hitting.rotation + owner.rotation);
            particle.position_data = gate::point_to_real_cells(graph, hitting.location, point + rotated_owner_pos);
            collision_points.push(Reverse(particle));
        }
    }
    CollisionObject::new(
        rotated_owner_pos,
        rel_velocity,
        owner.id,
        hitting.id,
        collision_points
    )
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
    fn cell_corner_mask<T:GraphNode>(graph:&SparseDirectedGraph<T>, start:ExternalPointer, zorder:ZorderPath) -> u8 {
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
                    let pointer = graph.read(start, &check_zorder.steps()).unwrap();
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

    pub fn tree_corners<T:GraphNode>(graph:&SparseDirectedGraph<T>, start:ExternalPointer) -> Vec<Corners> {
        let leaves = graph.dfs_leaf_cells(start);
        let mut corners = Vec::new();
        for cell in leaves {
            let zorder = ZorderPath::from_cell(cell.cell, start.height - cell.pointer.height);
            corners.push( Corners::new(
                cell_corners(cell),
                cell.pointer.pointer,
                if is_ignore(cell.pointer.pointer) { 0 } else { cell_corner_mask(graph, start, zorder) }
            ));
        }
        corners 
    }
    
    
}

//Rewrite all this
// fn determine_walls_hit(possibly_hit_walls:BVec2, velocity:Vec2, position_data:[Option<CellData>; 4]) -> Option<BVec2> {
//     let hit_walls = {
//         if possibly_hit_walls == BVec2::TRUE {
//             slide_check(velocity, position_data)
//         } else { possibly_hit_walls }
//     };
//     match hit_walls {
//         BVec2::FALSE => { None }
//         _ => { Some(hit_walls) }
//     }
// }
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
    let x_ignore = if let Some(pos_data) = position_data[x_slide_check] {
        is_ignore(pos_data.pointer.pointer)
    } else { true };
    let y_ignore = if let Some(pos_data) = position_data[y_slide_check] {
        is_ignore(pos_data.pointer.pointer)
    } else { true };
    BVec2::new(y_ignore, x_ignore )
}

fn check_wall(position_data:&[Option<CellData>; 4], idx: usize) -> CellType {
    match &position_data[idx] {
        Some(cell_data) => {
            let index = *cell_data.pointer.pointer;
            if index == 1 || index == 3 { CellType::Solid 
            } else { CellType::Air }
        }
        None => CellType::Void
    }
}

fn hitting_wall(position_data:[Option<CellData>; 4], velocity:Vec2) -> Option<BVec2> {
    if velocity == Vec2::ZERO { return None }

    let (indices, is_vertical) = get_relevant_indices(velocity);
    
    let is_solid = |idx: usize| -> bool {
        matches!(check_wall(&position_data, idx), CellType::Solid)
    };
    let result = if velocity.x == 0. || velocity.y == 0. {
        // Single-axis movement
        let wall = indices.iter().any(|&idx| is_solid(idx));
        if is_vertical {
            BVec2::new(false, wall)
        } else {
            BVec2::new(wall, false)
        }
    } else {
        // Diagonal movement - if the corner is solid, we're hitting in both directions
        let wall = is_solid(indices[0]);
        BVec2::new(wall, wall)
    };
    if result == BVec2::TRUE {
        Some(slide_check(velocity, position_data))
    } else if result != BVec2::FALSE { Some(result) } else { None }
}

fn is_ignore(index:Index) -> bool { *index == 2 || *index == 0 }
