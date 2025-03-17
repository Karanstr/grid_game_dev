use roots::{find_root_brent, SearchError, SimpleConvergency};
use macroquad::math::Vec2;
use crate::engine::math::*;

#[derive(derive_new::new)]
struct IterationTracker {
    #[new(value = "SimpleConvergency { eps: FP_EPSILON, max_iter: 10 }")]
    convergency: SimpleConvergency<f32>,
    #[new(value = "0")]
    iterations: usize,
}
impl roots::Convergency<f32> for IterationTracker {
    fn is_root_found(&mut self, y: f32) -> bool {
        self.convergency.is_root_found(y)
    }

    fn is_converged(&mut self, x1: f32, x2: f32) -> bool {
        self.convergency.is_converged(x1, x2)
    }

    fn is_iteration_limit_reached(&mut self, iter: usize) -> bool {
        self.iterations = iter;
        self.convergency.is_iteration_limit_reached(iter)
    }
}

pub struct Line {
    start: Vec2,
    end: Vec2,
    a: f32,
    b: f32,
    c: f32,
    norm: f32,
}
impl Line {
    pub fn new(start: Vec2, end: Vec2) -> Self {
        // Calculate line equation coefficients: Ax + By + C = 0
        let dx = end.x - start.x;
        let dy = end.y - start.y;
        
        // Calculate line equation coefficients
        let a = -dy;
        let b = dx;
        let c = dy * start.x - dx * start.y;
        let norm = (a * a + b * b).sqrt();
        
        Self { start, end, a, b, c, norm }
    }
    
    pub fn distance_to(&self, point: Vec2) -> f32 {
        (self.a * point.x + self.b * point.y + self.c) / self.norm
    }
    
    pub fn point_within_bounds(&self, point: Vec2) -> bool {
        point.clamp(self.start.min(self.end), self.start.max(self.end)).approx_eq(point)
    }
}

#[derive(Debug, Clone, Copy, derive_new::new)]
pub struct Motion {
    pub target_center: Vec2,
    pub owner_center: Vec2,
    pub offset_from_owner: Vec2,
    pub velocity: Vec2,
    pub target_angular: f32,
    pub owner_angular: f32,
}
impl Motion {

    pub fn project_to(self, ticks: f32) -> Vec2 {
        // https://www.desmos.com/calculator/l96dczj2s1 Calculations
        // https://www.desmos.com/calculator/wtvezmljqb Visualizations (target center forced to be (0,0))
        let rotation = Vec2::from_angle(ticks * self.owner_angular);
        let revolution = Vec2::from_angle(ticks * -self.target_angular);
        let orbit_point = self.offset_from_owner.rotate(rotation) + self.owner_center - self.target_center;
        (orbit_point + ticks * self.velocity).rotate(revolution) + self.target_center
    }

    pub fn solve(self, line: Line, max_time: f32) -> Option<f32> {
        let mut iter_tracker = IterationTracker::new();
        match find_root_brent(0., max_time,
            |t: f32| line.distance_to(self.project_to(t)),
            &mut iter_tracker
        ) {
            Ok(t) => {
                println!("Found in {} iterations", iter_tracker.iterations);
                Some(t)
            },
            Err(SearchError::NoConvergency) => panic!("Increase iterations"),
            Err(_) => None
        }
    }

}

#[test]
fn _manual_test() {
    let motion = Motion {
        target_center: Vec2::new(1., 2.),
        owner_center: Vec2::new(0., 2.),
        offset_from_owner: Vec2::new(1., 1.),
        velocity: Vec2::new(2., 0.),
        target_angular: 0.1,
        owner_angular: 1.,
    };
    dbg!(motion.project_to(1.0));
}
