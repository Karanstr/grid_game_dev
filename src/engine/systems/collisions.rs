use std::cmp::Ordering;
use super::*;


#[derive(Debug, Clone, new)]
pub struct Particle {
    pub position : Vec2,
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


/*
    pub fn apply_linear_acceleration(&mut self, acceleration:Vec2) { 
        self.velocity += acceleration; 
        self.remove_neglible_vel() 
    }

    fn remove_neglible_vel(&mut self) {
        let speed_min = 0.0005;
        if self.velocity.x.abs() < speed_min { self.velocity.x = 0. }
        if self.velocity.y.abs() < speed_min { self.velocity.y = 0. }
    }

    fn handle_resist(&mut self, impulse:Vec2, walls_hit:BVec2) {
        match self.collision_type {
            CollisionType::Static => { }
            CollisionType::Dynamic => {
                self.velocity += impulse * Vec2::from(walls_hit);
                if impulse.y < 0. && walls_hit.y { self.can_jump = true }
                self.remove_neglible_vel()
            }
        }
    }


    //Make this not bad?
    fn exposed_corners(&self, root:Root, zorder:ZorderPath) -> u8 {
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
                    if let Some(zorder) = zorder.move_cartesianly(offset) {
                        zorder
                    } else { continue }
                };
                for _ in 0 .. root.height - zorder.depth {
                    check_zorder = check_zorder.step_down(direction);
                }
                let path = check_zorder.steps();
                let (node_pointer, _) = self.graph.read(root.pointer, &path);
                if let Some(OnTouch::Resist(walls)) = self.index_collision(node_pointer.index) {
                    if walls != BVec2::TRUE { continue }
                    exposed_mask -= 1 << i;
                    break
                }
            }
        }
        exposed_mask
    }

    fn formatted_exposed_corners(&self, object_within:&Object, cur_pos: Vec2, owner:usize, hitting:usize) -> Vec<Particle> {
        let leaves = self.graph.dfs_leaves(object_within.root.pointer);
        let mut corners = Vec::new();
        for (zorder, index) in leaves {
            if !matches!(self.index_collision(index).unwrap_or(OnTouch::Ignore), OnTouch::Ignore) {
                let corner_mask = self.exposed_corners(object_within.root, zorder);
                let top_left_corner = object_within.cell_top_left_corner(zorder.to_cell(), zorder.depth) - object_within.position + cur_pos;
                let cell_length = object_within.cell_length(zorder.depth);
                for i in 0 .. 4 {
                    if corner_mask & 1 << i != 0 {
                        corners.push(Particle::new(
                            top_left_corner + cell_length * IVec2::new(i & 1, i >> 1).as_vec2(),
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
            let hitting_aabb = hitting.aabb().expand(velocity * multiplier);
            let point_aabb = AABB::new(corner.position, Vec2::ZERO).expand( velocity * multiplier);
            if hitting_aabb.intersects(point_aabb) != BVec2::TRUE { self.camera.outline_bounds(point_aabb, 2., RED); continue }
            else { self.camera.outline_bounds(point_aabb, 2., GREEN); }
            corner.position_data = hitting.get_data_at_position(&self, corner.position)[configured_direction(-velocity, corner.configuration)];
            corners.push(Reverse(corner));
        }
        corners
    }
   
    fn get_corners(&self, object1:&Object, object2:&Object, multiplier:f32, obj1_index:usize, obj2_index:usize) -> BinaryHeap<Reverse<Particle>> {
        let relative_velocity = object1.velocity - object2.velocity;
        let corners = [
            self.cull_and_fill_corners(object2, self.formatted_exposed_corners(object1, object1.position, obj1_index, obj2_index), relative_velocity, multiplier),
            self.cull_and_fill_corners(object1, self.formatted_exposed_corners(object2, object2.position, obj2_index, obj1_index), -relative_velocity, multiplier)
        ];
        BinaryHeap::from(corners.concat())
    }

    pub fn n_body_collisions(&mut self, multiplier:f32) {
        let mut ticks_into_projection = 0.;
        loop {
            let tick_max = 1. - ticks_into_projection;
            let mut corners = BinaryHeap::new();
            for i in 0 .. self.objects.len() {
                for j in i + 1 .. self.objects.len() { 
                    if within_range(&self.objects[i], &self.objects[j], multiplier, &self.camera) {
                        let (obj1_index, obj2_index) = (i, j);
                        corners.extend(self.get_corners(&self.objects[i], &self.objects[j], multiplier, obj1_index, obj2_index));
                    }
                }
            }
            let Some((action, ticks_at_hit, (owner, hitting))) = self.find_next_action(corners, tick_max) else {
                //No collision, move objects their remaining distance
                for object in self.objects.iter_mut() {
                    object.position += object.velocity * tick_max;
                }
                break
            };
            ticks_into_projection += ticks_at_hit;
            for object in self.objects.iter_mut() {
                object.position += object.velocity * ticks_at_hit;
            }
            if let OnTouch::Resist(walls_hit) = action {
                let relative_velocity = self.objects[owner].velocity - self.objects[hitting].velocity;
                let impulse = -(1. + 0.5)/2. * relative_velocity;
                self.objects[owner].handle_resist(impulse, walls_hit);
                self.objects[hitting].handle_resist(-impulse, walls_hit);
            }
        }
        
        let drag_multiplier = 0.1;
        for object in self.objects.iter_mut() {
            object.apply_linear_acceleration(-object.velocity * drag_multiplier);
        }
    }

    //Replace this return type with a struct
    fn find_next_action(&self, mut corners:BinaryHeap<Reverse<Particle>>, tick_max:f32) -> Option<(OnTouch, f32, (usize, usize))> {
        let objects = &self.objects;
        let mut action = OnTouch::Ignore;
        let mut ticks_to_hit = tick_max;
        let mut hit = false;
        let mut col_owner = 0;
        let mut col_hitting = 0;
        while let Some(mut cur_corner) = corners.pop().map(|x| x.0) {
            if cur_corner.ticks_into_projection >= ticks_to_hit { break }
            let (owner, hitting) = cur_corner.rel_objects;
            let relative_velocity = objects[owner].velocity - objects[hitting].velocity;
            let Some(hit_point) = self.next_intersection(cur_corner.position, relative_velocity, cur_corner.position_data, &objects[hitting]) else { continue };
            cur_corner.ticks_into_projection += hit_point.ticks_to_hit;
            if cur_corner.ticks_into_projection >= ticks_to_hit { continue }
            cur_corner.position = hit_point.position;
            let position_data = objects[hitting].get_data_at_position(&self, cur_corner.position);
            cur_corner.position_data = position_data[configured_direction(relative_velocity, cur_corner.configuration)];
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

    fn next_intersection(&self, position:Vec2, velocity:Vec2, position_data:Option<LimPositionData>, hitting:&Object) -> Option<f32> {
        let hitting_aabb = hitting.aabb();
        let top_left = hitting_aabb.min();
        let bottom_right = hitting_aabb.max();
        let within_bounds = hitting_aabb.contains(position);
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
        Some(ticks_to_hit)
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

    pub fn identify_object_region(&self, moving_object_index:usize, hitting_object_index:usize, multiplier:f32) {
        let moving_object = &self.objects[moving_object_index];
        let hitting_object = &self.objects[hitting_object_index];
        let bounding_box = moving_object.aabb().expand((moving_object.velocity - hitting_object.velocity) * multiplier);
        if hitting_object.aabb().intersects(bounding_box) != BVec2::TRUE { return }
        let top_left_zorder = {
            match hitting_object.get_data_at_position(&self, bounding_box.min())[0] {
                Some(data) => ZorderPath::from_cell(data.cell, data.depth),
                None => ZorderPath::root(),
            }
        };
        let bottom_right_zorder = {
            match hitting_object.get_data_at_position(&self, bounding_box.max())[0] {
                Some(data) => ZorderPath::from_cell(data.cell, data.depth),
                None => ZorderPath::root(),
            }
        };
        
        let parent_zorder = top_left_zorder.shared_parent(bottom_right_zorder);
        self.camera.outline_vec_rectangle(hitting_object.cell_top_left_corner(parent_zorder.to_cell(), parent_zorder.depth), hitting_object.cell_length(parent_zorder.depth), 4., GREEN);
    }

    pub fn expand_object_domain(&mut self, object_index:usize, direction:usize) {
        let object = &mut self.objects[object_index];
        //Prevent zorder overflow for now
        if object.root.height == 15 { dbg!("We don't overflow around here"); return }
        object.position += object.cell_length(0) * zorder_to_direction(direction as u32)/2.;
        let new_root = self.graph.set_node(NodePointer::new(Index(0)), &[direction as u32], object.root.pointer).unwrap();
        self.graph.swap_root(object.root.pointer, new_root);
        object.root.pointer = new_root;
        object.root.height += 1;
    }

    pub fn shrink_object_domain(&mut self, object_index:usize, preserve_direction:usize) {
        let object = &mut self.objects[object_index];
        if object.root.height == 0 { return }
        object.position += object.cell_length(0) * -zorder_to_direction(preserve_direction as u32)/4.;
        let new_root = self.graph.set_node(object.root.pointer, &[], self.graph.child(object.root.pointer, preserve_direction).unwrap()).unwrap();
        self.graph.swap_root(object.root.pointer, new_root);
        object.root.pointer = new_root;
        object.root.height -= 1;
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

pub fn within_range(object1:&Object, object2:&Object, multiplier:f32, camera:&Camera) -> bool {
    let obj1_aabb = object1.aabb().expand(object1.velocity * multiplier);
    let obj2_aabb = object2.aabb().expand(object2.velocity * multiplier);
    camera.outline_bounds(obj1_aabb, 2., RED);
    camera.outline_bounds(obj2_aabb, 2., RED);
    obj1_aabb.intersects(obj2_aabb) == BVec2::TRUE
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

pub fn zorder_to_direction(zorder:u32) -> Vec2 {
    -Vec2::new(
        if zorder & 0b1 == 0b1 { 1. } else { -1. },
        if zorder & 0b10 == 0b10 { 1. } else { -1. },
    )
}
*/