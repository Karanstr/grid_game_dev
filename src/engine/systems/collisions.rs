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

//Decide whether to remember particle velocity
#[derive(Debug, Clone, new)]
pub struct Particle {
    pub position : Vec2,
    pub velocity : Vec2,
    #[new(value = "0.")]
    pub ticks_into_projection : f32,
    #[new(value = "None")]
    pub position_data : Option<CellData>,
    pub configuration : Configurations,
    pub owner : ID,
    pub hitting : ID,
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

#[derive(Debug, Clone)]
enum Action {
    Resist(BVec2),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Configurations {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight
}
impl Configurations {
    pub fn from_index(index:usize) -> Self {
        match index {
            0 => Self::TopLeft,
            1 => Self::TopRight,
            2 => Self::BottomLeft,
            3 => Self::BottomRight,
            _ => panic!("Invalid Configuration Index")
        }
    }
}

#[derive(Debug, Clone)]
struct Hit {
    pub owner : ID,
    pub hitting : ID,
    pub action : Action
}

pub fn n_body_collisions<T:GraphNode>(entities:&mut EntityPool, graph:&SparseDirectedGraph<T>, camera:&Camera, static_thing:ID) {
    let mut tick_max = 1.;
    loop {
        let mut corners = BinaryHeap::new();
        for idx in 0 .. entities.entities.len() {
            let entity = &entities.entities[idx];
            for other_idx in idx + 1 .. entities.entities.len() {
                let other = &entities.entities[other_idx];
                if within_range(&entity, &other, camera) {
                    corners.extend(particles(graph, &entity, &other, &camera));
                }
            }
        }
        
        let (hits, ticks_to_action) = find_next_action(&entities, &graph, corners, tick_max);
        // Update positions with error checking
        for entity in &mut entities.entities {
            let delta = entity.velocity * ticks_to_action;
            // Skip tiny movements that could cause precision issues
            if !vec2_approx_eq(delta, Vec2::ZERO) { entity.location.position += delta;}
        }
        
        tick_max -= ticks_to_action;
        if hits.is_empty() { break }
        
        for hit in hits {
            match hit.action {
                Action::Resist(walls_hit) => {
                    let hitting = entities.get_entity(hit.hitting).unwrap();
                    let world_to_hitting = Mat2::from_angle(-hitting.rotation);
                    
                    let walls_as_int = IVec2::from(walls_hit).as_vec2();
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
            }
        }
    }
    let drag_multiplier = 0.95;
    for entity in &mut entities.entities { 
        entity.velocity *= drag_multiplier;
        entity.velocity = vec2_remove_err(entity.velocity);
    }

}

fn find_next_action<T:GraphNode>(entities:&EntityPool, graph:&SparseDirectedGraph<T>, mut corners:BinaryHeap<Reverse<Particle>>, tick_max:f32) -> (Vec<Hit>, f32) {
    let mut ticks_to_action = tick_max;
    // dbg!(&corners);
    if corners.is_empty() { return (Vec::new(), ticks_to_action) }
    let mut actions = Vec::new();
    while let Some(Reverse(mut cur_corner)) = corners.pop() {
        if cur_corner.ticks_into_projection >= ticks_to_action { break }
        let hitting_location = entities.get_entity(cur_corner.hitting).unwrap().location;
        let Some(ticks_to_hit) = next_intersection(
            cur_corner.position,
            cur_corner.velocity,
            cur_corner.position_data,
            hitting_location
        ) else { continue };
        if ticks_to_hit < -ticks_to_action { continue }
        cur_corner.ticks_into_projection += ticks_to_hit;
        if cur_corner.ticks_into_projection >= ticks_to_action { continue }
        cur_corner.position += cur_corner.velocity * ticks_to_hit;
        let all_data = gate::point_to_real_cells(graph, hitting_location, cur_corner.position);
        cur_corner.position_data = all_data[configured_direction(cur_corner.velocity, cur_corner.configuration)];
        if cur_corner.position_data.is_none() { continue }
        if is_ignore(cur_corner.position_data.unwrap().pointer.pointer) { }
        else if let Some(hit_walls) = determine_walls_hit(BVec2::TRUE, cur_corner.velocity, cur_corner.configuration, all_data) {
            actions.clear();
            actions.push(
                Hit {
                    owner : cur_corner.owner,
                    hitting : cur_corner.hitting,
                    action : Action::Resist(hit_walls)
                }
            );
            ticks_to_action = cur_corner.ticks_into_projection;
            continue
        }
        corners.push(Reverse(cur_corner));
    }
    (actions, ticks_to_action)
}

fn next_intersection(point:Vec2, velocity:Vec2, position_data:Option<CellData>, hitting_location:Location) -> Option<f32> {
    let hitting_aabb = bounds::aabb(hitting_location.position, hitting_location.pointer.height);
    let top_left = hitting_aabb.min();
    let bottom_right = hitting_aabb.max();
    let within_bounds = hitting_aabb.contains(point);
    let (cell, height) = match position_data {
        Some(data) => { (data.cell.as_vec2(), data.pointer.height) }
        None => {
            let mut cell = Vec2::ZERO;
            if point.x <= top_left.x {
                if velocity.x > 0. { cell.x = -1. } else { return None }
            } else if point.x >= bottom_right.x {
                if velocity.x < 0. { cell.x = 1. } else { return None }
            }
            if point.y <= top_left.y {
                if velocity.y > 0. { cell.y = -1. } else { return None }
            } else if point.y >= bottom_right.y {
                if velocity.y < 0. { cell.y = 1. } else { return None }
            }
            (cell, hitting_location.pointer.height)
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
    if ticks_to_hit.is_nan() || ticks_to_hit.is_infinite() { None }
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

pub fn particles<T:GraphNode>(graph:&SparseDirectedGraph<T>, object1:&Entity, object2:&Entity, camera:&Camera ) -> BinaryHeap<Reverse<Particle>> {
    let mut result = BinaryHeap::from(corner_handling::actionable_corners(graph, object1, object2, camera));
    result.extend(corner_handling::actionable_corners(graph, object2, object1, camera));
    result
}

#[derive(Debug, new)]
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
    
    //Should be able to cull even more based on hang_check principle
    pub fn actionable_corners<T:GraphNode>(graph:&SparseDirectedGraph<T>, owner:&Entity, hitting:&Entity, camera:&Camera) -> Vec<Reverse<Particle>> {
        let mut culled_corners = Vec::new();
        
        let align_hitting = Vec2::from_angle(-hitting.rotation);
        let offset = bounds::center_to_edge(owner.location.pointer.height);
        
        let rel_velocity = vec2_remove_err((owner.velocity - hitting.velocity).rotate(align_hitting));
        
        // let hitting_aabb = bounds::aabb(hitting.location.position, hitting.location.pointer.height);
        let aligned_owner_pos = (owner.location.position - hitting.location.position).rotate(align_hitting) + hitting.location.position;
        
        camera.draw_point(aligned_owner_pos, 0.1, GREEN);
        
        for corners in owner.corners.iter() {
            for i in 0..4 {
                if corners.mask & (1 << i) == 0 { continue }
                
                let point = (corners.points[i] - offset).rotate(align_hitting).rotate(owner.forward) + aligned_owner_pos;
                
                camera.draw_point(point, 0.1, RED);
                
                let configuration = Configurations::from_index(i);
                
                if hittable_walls(rel_velocity, configuration) == BVec2::FALSE { continue }
                
                // let point_aabb = AABB::new(point, Vec2::splat(EPSILON)).expand(rel_velocity);
                // if hitting_aabb.intersects(point_aabb) != BVec2::TRUE { continue }
                
                let mut particle = Particle::new(point, rel_velocity, configuration, owner.id, hitting.id);
                if let Some(smallest_cell) = gate::point_to_cells(hitting.location, 0, point)[configured_direction(-rel_velocity, configuration)] {
                    particle.position_data = Some(gate::find_real_cell(graph, hitting.location.pointer, smallest_cell));
                }
                culled_corners.push(Reverse(particle));
            }
        }
        culled_corners
    }

    
}

fn determine_walls_hit(possibly_hit_walls:BVec2, velocity:Vec2, configuration:Configurations, position_data:[Option<CellData>; 4]) -> Option<BVec2> {
    let hit_walls = {
        let potential_hits = possibly_hit_walls & hittable_walls(velocity, configuration);
        if potential_hits == BVec2::TRUE {
            slide_check(velocity, position_data)
        } else { potential_hits }
    };
    match hit_walls {
        BVec2::TRUE => { Some(mag_slide_check(velocity)) }
        BVec2::FALSE => { None }
        _ => { Some(hit_walls) }
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
    let x_ignore = if let Some(pos_data) = position_data[x_slide_check] {
        is_ignore(pos_data.pointer.pointer)
    } else { true };
    let y_ignore = if let Some(pos_data) = position_data[y_slide_check] {
        is_ignore(pos_data.pointer.pointer)
    } else { true };
    BVec2::new(y_ignore, x_ignore )
}

fn is_ignore(index:Index) -> bool {
    *index == 0 || *index == 2
}

//Compares configuration to velocity to determine which walls you're allowed to hit
//(A corner on the top of a block can't hit a wall below it, the block is in the way)
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

//Uses configuration to decide which cell you're hitting while moving along a single axis.
//We can assume a corner would've been culled if it is irrelevant
//This meaning we only care about corners on the y wall and the x wall (This isn't true..?)
pub fn configured_direction(direction:Vec2, configuration:Configurations) -> usize {
    if direction == Vec2::ZERO { dbg!("AHHH"); }
    let clamped: Vec2 = direction.signum().max(Vec2::ZERO);
    if direction.x == 0. {
        2 * clamped.y as usize | if configuration == Configurations::TopLeft || configuration == Configurations::BottomLeft { 1 } else { 0 }
    } else if direction.y == 0. {
        clamped.x as usize | 2 * if configuration == Configurations::TopLeft || configuration == Configurations::TopRight { 1 } else { 0 }
    } else {
        2 * clamped.y as usize | clamped.x as usize
    }
}
