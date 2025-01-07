use std::cmp::{Reverse, Ordering};
use std::collections::BinaryHeap;

use super::*;

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
    pub owner : Entity,
    pub hitting : Entity,
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
    Ignore,
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
    pub owner : Entity,
    pub hitting : Entity,
    pub ticks_into_projection : f32,
    pub action : Action
}

pub struct CollisionSystem;
impl CollisionSystem {
    pub fn n_body_collisions(game_data:&mut GameData) {
        let mut ticks_into_projection = 0.;
        loop {
            let tick_max = 1. - ticks_into_projection;
            let mut corners = BinaryHeap::new();
            for (entity, (location, velocity)) in game_data.entities.query::<(&Location, &Velocity)>().iter() {
                for (entity2, (location2, velocity2)) in game_data.entities.query::<(&Location, Option<&Velocity>)>().iter() {
                    if entity == entity2 || velocity.0 == Vec2::ZERO { continue }
                    if Self::within_range(&location, &location2, velocity, velocity2.unwrap_or(&Velocity(Vec2::ZERO)), &game_data.camera) {
                        corners.extend(Self::particles(&game_data.graph, (entity, &location, velocity.0), (entity2, &location2, velocity2.unwrap_or(&Velocity(Vec2::ZERO)).0), &game_data.camera));
                    }
                }
            }
            let (hits, ticks_to_action) = Self::find_next_action(&mut game_data.entities, &game_data.graph, corners, tick_max);
            for (_, (location, velocity)) in game_data.entities.query::<(&mut Location, &Velocity)>().iter() {
                location.position += velocity.0 * ticks_to_action;
            }
            ticks_into_projection += ticks_to_action;
            if ticks_into_projection == 1. { break }
            for hit in hits {
                dbg!(hit.clone());
                match hit.action {
                    Action::Ignore => {}
                    Action::Resist(walls_hit) => {
                        let walls_as_int = IVec2::from(walls_hit).as_vec2();
                        let owner_velocity = game_data.entities.query_one_mut::<&mut Velocity>(hit.owner).unwrap_or(&mut Velocity(Vec2::ZERO)).0;
                        let hitting_velocity = game_data.entities.query_one_mut::<&mut Velocity>(hit.hitting).unwrap_or(&mut Velocity(Vec2::ZERO)).0;
                        let relative_velocity = owner_velocity - hitting_velocity;
                        let impulse = -(1. + 0.5)/2. * relative_velocity;
                        if let Ok(velocity) = game_data.entities.query_one_mut::<&mut Velocity>(hit.owner) {
                            velocity.0 += impulse * walls_as_int;
                        }
                        if let Ok(velocity) = game_data.entities.query_one_mut::<&mut Velocity>(hit.hitting) {
                            velocity.0 -= impulse * walls_as_int;
                        }
                    }
                }
            }
        }
        let drag_multiplier = 0.9;
        for (_, velocity) in game_data.entities.query::<&mut Velocity>().iter() {
            velocity.0 *= drag_multiplier;
            if velocity.0.length() < 0.0001 { velocity.0 = Vec2::ZERO }
        }
    }

    fn find_next_action(entities:&mut World, graph:&SparseDirectedGraph, mut corners:BinaryHeap<Reverse<Particle>>, tick_max:f32) -> (Vec<Hit>, f32) {
        let mut ticks_to_action = tick_max;
        let mut actions = Vec::new(); 
        while let Some(Reverse(mut cur_corner)) = corners.pop() {
            if cur_corner.ticks_into_projection >= ticks_to_action { continue }
            let hitting_location = entities.query_one_mut::<&Location>(cur_corner.hitting).unwrap();
            let Some(ticks_to_hit) = Self::next_intersection(
                cur_corner.position,
                cur_corner.velocity,
                cur_corner.position_data,
                hitting_location
            ) else { continue };
            cur_corner.ticks_into_projection += ticks_to_hit;
            if cur_corner.ticks_into_projection >= ticks_to_action { continue }
            cur_corner.position += cur_corner.velocity * ticks_to_hit;
            let all_data = Gate::point_to_real_cells(graph, hitting_location, cur_corner.position);
            cur_corner.position_data = all_data[configured_direction(cur_corner.velocity, cur_corner.configuration)];
            if cur_corner.position_data.is_none() { continue }
            if is_ignore(cur_corner.position_data.unwrap().pointer.pointer) { }
            else if let Some(hit_walls) = determine_walls_hit(BVec2::TRUE, cur_corner.velocity, cur_corner.configuration, all_data) {
               if cur_corner.ticks_into_projection != ticks_to_action { actions.clear() }
               actions.push(
                    Hit {
                        owner : cur_corner.owner,
                        hitting : cur_corner.hitting,
                        ticks_into_projection : cur_corner.ticks_into_projection,
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

    fn next_intersection(point:Vec2, velocity:Vec2, position_data:Option<CellData>, hitting_location:&Location) -> Option<f32> {
        let hitting_aabb = Bounds::aabb(hitting_location.position, hitting_location.pointer.height);
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
        let cell_length = Bounds::cell_length(height);
        let boundary_corner = top_left + cell * cell_length + cell_length * quadrant;
        
        let ticks = ((boundary_corner - point) / velocity).abs(); 
        let ticks_to_hit = match (within_bounds.x, within_bounds.y) {
            (false, false) => { ticks.max_element() },
            (true, false) if ticks.x == 0. => { ticks.y },
            (false, true) if ticks.y == 0. => { ticks.x },
            _ => { ticks.min_element() },
        };
        if ticks_to_hit.is_nan() || ticks_to_hit.is_infinite() { return None }
        Some(ticks_to_hit)
    }

    pub fn within_range(location:&Location, location2:&Location, velocity:&Velocity, velocity2:&Velocity, camera:&Camera) -> bool { 
        let aabb = Bounds::aabb(location.position, location.pointer.height).expand(velocity.0);
        let aabb2 =  Bounds::aabb(location2.position, location2.pointer.height).expand(velocity2.0);
        camera.outline_bounds(aabb, 2., RED);
        camera.outline_bounds(aabb2, 2., RED);
        aabb.intersects(aabb2) == BVec2::TRUE
    }

    pub fn particles(graph:&SparseDirectedGraph, object1:(Entity, &Location, Vec2), object2:(Entity, &Location, Vec2), camera:&Camera ) -> BinaryHeap<Reverse<Particle>> {
        let all_corners1 = CornerHandling::tree_corners(graph, object1.1.pointer);
        let all_corners2 = CornerHandling::tree_corners(graph, object2.1.pointer);
        let mut result = BinaryHeap::from(CornerHandling::cull_and_fill_corners(graph, all_corners1, object1, object2, camera));
        result.extend(CornerHandling::cull_and_fill_corners(graph, all_corners2, object2, object1, camera));
        result
    }

}

#[derive(Debug, new)]
struct Corner {
    pub position:Vec2,
    pub configuration:Configurations,
}
impl Corner {
    pub fn to_particle(&self, velocity:Vec2, owner:Entity, hitting:Entity) -> Particle {
        Particle::new(self.position, velocity, self.configuration, owner, hitting)
    }
}
struct CornerHandling;
impl CornerHandling {
    pub fn cell_corner_mask(graph:&SparseDirectedGraph, start:ExternalPointer, zorder:ZorderPath) -> u8 {
        let mut exposed_mask = 0b1111;
        //Replace these with Configurations?
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
                let Some(mut check_zorder) = zorder.move_cartesianly(offset) else { continue };
                for _ in 0 .. start.height - zorder.depth {
                    check_zorder = check_zorder.step_down(direction);
                }
                let pointer = graph.read(start, &check_zorder.steps());
                //Add proper block lookup
                match *pointer.pointer.index {
                    0 | 2 => {},
                    _ => {
                            exposed_mask -= 1 << i;
                            break
                        }
                    }
                }
            }
        exposed_mask
    }

    pub fn tree_corners(graph:&SparseDirectedGraph, start:ExternalPointer) -> Vec<Corner> {
        let leaves = graph.dfs_leaves(start);
        let mut corners = Vec::new();
        for cell in leaves {
            if is_ignore(cell.pointer.pointer) { continue }
            let zorder = ZorderPath::from_cell(cell.cell, start.height - cell.pointer.height);
            let corner_mask = Self::cell_corner_mask(graph, start, zorder);
            let top_left_corner = Bounds::top_left_corner(cell.cell, cell.pointer.height);
            let cell_length = Bounds::cell_length(cell.pointer.height);
            for i in 0 .. 4 {
                if corner_mask & 1 << i != 0 {
                    corners.push(
                        Corner::new(top_left_corner + cell_length * IVec2::new(i & 1, i >> 1).as_vec2(), Configurations::from_index(i as usize))
                    );
                }
            }
        }
        corners
    }

    //Should be able to cull even more based on hang_check principle. (Please fact check when not hungry)
    pub fn cull_and_fill_corners(graph:&SparseDirectedGraph, corners:Vec<Corner>, hitting:(Entity, &Location, Vec2), owner:(Entity, &Location, Vec2), camera:&Camera) -> Vec<Reverse<Particle>> {
        let mut culled_corners = Vec::new();
        let hitting_aabb = Bounds::aabb(hitting.1.position, hitting.1.pointer.height).expand(hitting.2);
        let rel_velocity = owner.2 - hitting.2;
        if rel_velocity.length() == 0. { return culled_corners }
        for corner in corners.into_iter() {
            let hittable = hittable_walls(rel_velocity, corner.configuration);
            if hittable == BVec2::FALSE { continue }
            let point_aabb = AABB::new(corner.position, Vec2::ZERO).expand(owner.2);
            if hitting_aabb.intersects(point_aabb) != BVec2::TRUE { camera.outline_bounds(point_aabb, 2., RED); continue }
            else { camera.outline_bounds(point_aabb, 2., GREEN); }
            let mut particle = corner.to_particle(rel_velocity,owner.0, hitting.0);
            if let Some(smallest_cell) = Gate::point_to_cells(hitting.1, 0, corner.position)[configured_direction(-owner.2, corner.configuration)] {
                particle.position_data = Some(Gate::find_real_cell(graph, hitting.1.pointer, smallest_cell));
            }
            camera.draw_vec_rectangle(corner.position, Vec2::splat(0.1), ORANGE);
            culled_corners.push(Reverse(particle));
        }
        culled_corners
    }

}
   
fn determine_walls_hit(possibly_hit_walls:BVec2, velocity:Vec2, configuration:Configurations, position_data:[Option<CellData>; 4]) -> Option<BVec2> {
    let hit_walls = {
        let potential_hits = possibly_hit_walls & hittable_walls(velocity, configuration);
        if potential_hits == BVec2::TRUE {
            slide_check(velocity, position_data)
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

fn slide_check(velocity:Vec2, position_data:[Option<CellData>; 4]) -> BVec2 {
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
    let x_ignore = if let Some(pos_data) = position_data[x_slide_check] {
        is_ignore(pos_data.pointer.pointer)
    } else { true };
    let y_ignore = if let Some(pos_data) = position_data[y_slide_check] {
        is_ignore(pos_data.pointer.pointer)
    } else { true };
    BVec2::new(y_ignore, x_ignore )
}

fn is_ignore(pointer:InternalPointer) -> bool {
    *pointer.index == 0 || *pointer.index == 2
}

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

pub fn configured_direction(direction:Vec2, configuration:Configurations) -> usize {
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

//Why negative?
// pub fn zorder_to_direction(zorder:u32) -> Vec2 {
//     -Vec2::new(
//         if zorder & 0b1 == 0b1 { 1. } else { -1. },
//         if zorder & 0b10 == 0b10 { 1. } else { -1. },
//     )
// }
