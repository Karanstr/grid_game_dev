use super::*;

pub mod grid {
    use super::*;
    pub const MIN_CELL_LENGTH: Vec2 = Vec2::splat(1.);
    //Value loosely tuned to prevent both phasing and catching on corners
    //Used to sample area around a point to determine what cell(s) it's in
    pub const LIM_OFFSET: f32 = 2. / 0xFFF as f32;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct ZorderPath {
        pub zorder : u32,
        pub depth : u32
    }
    impl ZorderPath {
        pub fn root() -> Self {
            Self { zorder: 0, depth: 0 }
        }

        pub fn to_cell(&self) -> UVec2 {
            let mut cell = UVec2::ZERO;
            for layer in 0 .. self.depth {
                cell.x |= (self.zorder >> (2 * layer) & 0b1) << layer;
                cell.y |= (self.zorder >> (2 * layer + 1) & 0b1) << layer;
            }
            cell
        }

        pub fn from_cell(cell:UVec2, depth:u32) -> Self {
            let mut zorder = 0;
            for layer in (0 .. depth).rev() {
                let step = (((cell.y >> layer) & 0b1) << 1 ) | ((cell.x >> layer) & 0b1);
                zorder = (zorder << 2) | step;
            }
            Self { zorder, depth }
        }

        pub fn with_depth(&self, depth:u32) -> Self {
            let mut zorder = self.zorder;   
            if self.depth < depth {
                zorder <<= 2 * (depth - self.depth);
            } else {
                zorder >>= 2 * (self.depth - depth);
            };
            Self { zorder, depth }
        }

        pub fn move_cartesianly(&self, offset:IVec2) -> Option<Self> {
            let cell = self.to_cell();
            let end_cell = cell.as_ivec2() + offset;
            if end_cell.min_element() < 0 || end_cell.max_element() >= 2u32.pow(self.depth) as i32 {
                return None
            }
            Some(Self::from_cell(UVec2::new(end_cell.x as u32, end_cell.y as u32), self.depth))
        }

        pub fn read_step(&self, layer:u32) -> u32 {
            self.with_depth(layer).zorder & 0b11
        }

        #[allow(dead_code)]
        pub fn shared_parent(&self, other: Self) -> Self {
            let common_depth = u32::max(self.depth, other.depth);
            let a_zorder = self.with_depth(common_depth);
            let b_zorder = other.with_depth(common_depth);
            for layer in (0 ..= common_depth).rev() {
                if a_zorder.with_depth(layer) == b_zorder.with_depth(layer) {
                    return a_zorder.with_depth(layer)
                }
            }
            Self { zorder: 0, depth: 0 }
        }

        pub fn step_down(&self, direction:u32) -> Self {
            Self { 
                zorder: self.zorder << 2 | direction, 
                depth: self.depth + 1 
            }
        }

        pub fn steps(&self) -> Vec<u32> {
            let mut steps:Vec<u32> = Vec::with_capacity(self.depth as usize);
            for layer in 1 ..= self.depth {
                steps.push(self.read_step(layer));
            }
            steps
        }

        #[allow(dead_code)]
        pub fn cells_intersecting_aabb(_aabb:Aabb, _max_depth: u32) -> Vec<(u32, u32)> {
            todo!()
        }

    }

    #[derive(Debug, Clone, Copy, new)]
    pub struct CellData {
        pub pointer : ExternalPointer,
        pub cell : UVec2,
    }
    impl CellData {
        pub fn bound_data(&self) -> (Vec2, u32) { (self.cell.as_vec2(), self.pointer.height) }
    }

    pub mod bounds {
        use super::*;

        pub fn cell_length(height:u32) -> Vec2 {
            MIN_CELL_LENGTH * 2_f32.powi(height as i32 + 1)
        }

        pub fn center_to_edge(height:u32) -> Vec2 {
            cell_length(height) / 2.
        }

        pub fn aabb(position:Vec2, height:u32) -> Aabb {
            Aabb::new(position, center_to_edge(height))
        }

    }

    pub mod gate {
        use super::*;
        pub fn point_to_cells(grid_location:Location, height:u32, point:Vec2) -> [Option<UVec2>; 4]{
            let mut surrounding = [None; 4];
            let grid_length = bounds::cell_length(grid_location.pointer.height);
            let cell_length = bounds::cell_length(height);
            let origin_position = point - (grid_location.position - grid_length / 2.);
            let directions = [
                Vec2::new(-1., -1.),
                Vec2::new(1., -1.),
                Vec2::new(-1., 1.),
                Vec2::new(1., 1.),
            ];
            for i in 0 .. 4 {
                let cur_point = origin_position + LIM_OFFSET * directions[i];
                if cur_point.clamp(Vec2::ZERO, grid_length).approx_eq(cur_point) {
                    surrounding[i] = Some( (cur_point / cell_length).floor().as_uvec2() )
                }
            }
            surrounding
        }
        
        pub fn point_to_real_cells(grid_location:Location, point:Vec2) -> [Option<CellData>; 4] {
            let mut surrounding = [None; 4];
            let cells = point_to_cells(grid_location, 0, point);
            for i in 0..4 {
                if let Some(cell) = cells[i] {
                    surrounding[i] = Some(find_real_cell(grid_location.pointer, cell));
                }
            }
            surrounding
        }
        
        //Only works if cell is at height 0
        pub fn find_real_cell(start:ExternalPointer, cell:UVec2) -> CellData {
            let path = ZorderPath::from_cell(cell, start.height);
            let pointer = GAME_STATE.graph.read().read(start, &path.steps()).unwrap();
            let zorder = path.with_depth(start.height - pointer.height);
            CellData::new(pointer, zorder.to_cell())
        }

    }

    impl<T> SparseDirectedGraph<T> where T : GraphNode {
        pub fn dfs_leaf_cells(&self, start:ExternalPointer) -> Vec<CellData> {
            let mut stack = Vec::from([(start.pointer, ZorderPath::root())]);
            let mut leaves = Vec::new();
            while let Some((pointer, zorder)) = stack.pop() {
                if self.is_leaf(pointer) {
                    leaves.push(CellData::new(ExternalPointer::new(pointer, start.height - zorder.depth), zorder.to_cell()));
                } else { for i in 0 .. 4 {
                        let children = self.node(pointer).unwrap().children();
                        stack.push((children[i], zorder.step_down(i as u32)));
                    }
                }
            }
            leaves
        }
    }

}


#[derive(Debug, Clone, Copy, new)]
pub struct Aabb {
    center: Vec2,
    radius: Vec2
}
impl Aabb {

    pub fn min(&self) -> Vec2 { self.center - self.radius }
    pub fn max(&self) -> Vec2 { self.center + self.radius }

    pub fn center(&self) -> Vec2 { self.center }
    pub fn radius(&self) -> Vec2 { self.radius }

    pub fn intersects(&self, other:Self) -> BVec2 {
        (other.center - self.center).abs().less_eq(self.radius + other.radius)
    }
    pub fn contains(&self, point:Vec2) -> BVec2 {
        (point - self.center).abs().less_eq(self.radius)
    }
    
    pub fn move_by(&mut self, displacement:Vec2) { self.center += displacement }
    pub fn move_to(&mut self, position:Vec2) { self.center = position }
    
    pub fn expand(&self, distance:Vec2) -> Self {
        Self {
            center: self.center + distance / 2.,
            radius: self.radius + distance.abs() / 2.,
        }
    }
    pub fn shrink(&self, distance:Vec2) -> Self {
        Self {
            center: self.center - distance / 2.,
            radius: (self.radius - distance.abs() / 2.).abs(),
        }
    }

    pub fn exterior_will_intersect(&self, point:Vec2, velocity:Vec2) -> Option<Vec2> {
        let mut walls_will_hit = Vec2::ZERO;
        let top_left = self.min();
        let bottom_right = self.max();

        if point.x.less_eq(top_left.x) {
            if velocity.x.greater(0.) { walls_will_hit.x = -1. } else { return None }
        } else if point.x.greater_eq(bottom_right.x) {
            if velocity.x.less(0.) { walls_will_hit.x = 1. } else { return None }
        }

        // Check y-axis boundaries
        if point.y.less_eq(top_left.y) {
            if velocity.y.greater(0.) { walls_will_hit.y = -1. } else { return None }
        } else if point.y.greater_eq(bottom_right.y) {
            if velocity.y.less(0.) { walls_will_hit.y = 1. } else { return None }
        }
        
        Some(walls_will_hit)
    }
}
