pub use intersection::*;

mod intersection {
    use derive_new::new;
    use roots::{find_root_brent, SimpleConvergency};
    use macroquad::math::Vec2;
    use crate::engine::math::*;

    struct IterationTracker {
        convergency: SimpleConvergency<f32>,
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
    impl Default for IterationTracker {
        fn default() -> Self {
            Self {
                convergency: SimpleConvergency { eps: FP_EPSILON, max_iter: 20 },
                iterations: 0,
            }
        }
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Line {
        Vertical(f32),
        Horizontal(f32),
    }

    #[derive(Debug, Clone, Copy, new)]
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
            // https://www.desmos.com/calculator/l96dczj2s1
            let rotation = Vec2::from_angle(ticks * self.owner_angular);
            let revolution = Vec2::from_angle(ticks * -self.target_angular);
            let orbit_point = self.offset_from_owner.rotate(rotation) + self.owner_center - self.target_center;
            (orbit_point + ticks * self.velocity).rotate(revolution) + self.target_center
        }

        pub fn solve_all(self, line: Line, max_time: f32) -> Option<f32> {
        
            let (target, min_time_needed, index) = match line {
                Line::Vertical(x) => (x, 0., 0),
                Line::Horizontal(y) => (y, 0., 1),
            };

            let f = |t: f32| target - self.project_to(t)[index];
            
            let check_range = |t_start: f32, t_end: f32| -> Option<f32> {
                let mut iter_tracker = IterationTracker::default();
                if let Ok(t) = find_root_brent(t_start, t_end, &f, &mut iter_tracker) {
                    println!("Found in {} iterations", iter_tracker.iterations);
                    return Some(t)
                }
                None
            };

            /* Idk how to convert this correctly yet
            // Calculate the minimum time needed to potentially reach the target based on linear motion and rotation radius
            let radius = self.offset.length();
            let min_time_needed = if radius.greater_eq_mag(target_offset) { 0. } else {
                (target_offset.abs() - radius) / velocity
            };
            if min_time_needed.greater(max_time) { return None }
            
            let rotation_period = 2. * PI / self.angular_velocity.abs();
            
            check_range(min_time_needed, (min_time_needed + rotation_period).min(max_time))
            */
            check_range(min_time_needed, max_time)
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

}
