pub use intersection::*;

mod intersection {
    use derive_new::new;
    use roots::{find_root_brent, SimpleConvergency};
    use macroquad::math::Vec2;
    use std::f32::consts::PI;
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
        Vertical(f32),   // x = value
        Horizontal(f32), // y = value
    }

    #[derive(Clone, Copy, new)]
    pub struct Motion {
        pub center_of_rotation: Vec2,
        pub offset: Vec2,
        pub velocity: Vec2,
        pub angular_velocity: f32,
    }
    impl Motion {

        pub fn first_intersection(self, line: Line, max_time: f32) -> Option<f32> {
            match (self.velocity.is_zero(), self.angular_velocity.is_zero()) {
                (true, true) => None,
                (false, true) => self.solve_pure_linear(line, max_time),
                (true, false) => self.solve_pure_rotation(line, max_time),
                (false, false) => self.solve_linear_and_rotation(line, max_time),
            }
        }

        fn solve_linear_and_rotation(self, line: Line, max_time: f32) -> Option<f32> {
            let (target, center, velocity, offset_parallel, offset_perp) = match line {
                Line::Vertical(x) => (x, self.center_of_rotation.x, self.velocity.x, self.offset.x, -self.offset.y),
                Line::Horizontal(y) => (y, self.center_of_rotation.y, self.velocity.y, self.offset.y, self.offset.x),
            };

            // https://www.desmos.com/calculator/8iqoshfwcg
            let f = |t: f32| {
                let angle = t * self.angular_velocity;
                let (sin_t, cos_t) = angle.sin_cos();
                center + t*velocity + offset_parallel*cos_t + offset_perp*sin_t - target
            };
            
            let check_range = |t_start: f32, t_end: f32| -> Option<f32> {
                let f_start = f(t_start);
                let f_end = f(t_end);
                
                if f_start * f_end <= 0.0 {
                    let mut iter_tracker = IterationTracker::default();
                    if let Ok(t) = find_root_brent(t_start, t_end, &f, &mut iter_tracker) {
                        println!("Found in {} iterations", iter_tracker.iterations);
                        return Some(t);
                    }
                }
                None
            };

            let radius = self.offset.length();
            
            // Calculate the minimum time needed to potentially reach the target based on linear motion and rotation radius
            let min_time_needed = (((target-center).abs() - radius) / velocity.abs()).max(0.);
            
            if min_time_needed.greater(max_time) { return None }
            
            // Calculate time for a complete rotation
            let rotation_period = 2.0 * PI / self.angular_velocity.abs();
            check_range(min_time_needed, min_time_needed + rotation_period)
            
        }

        fn solve_pure_linear(self, line: Line, max_time: f32) -> Option<f32> {
            let (target, center, velocity, offset) = match line {
                Line::Vertical(x) => (x, self.center_of_rotation.x, self.velocity.x, self.offset.x),
                Line::Horizontal(y) => (y, self.center_of_rotation.y, self.velocity.y, self.offset.y),
            };

            let point = center + offset;
            let t = (target - point) / velocity;
            (t.greater(0.) && t.less_eq(max_time)).then_some(t)
        }

        fn solve_pure_rotation(self, line: Line, max_time: f32) -> Option<f32> {
            let (target, center_pos, offset_parallel, offset_perp) = match line {
                Line::Vertical(x) => (x, self.center_of_rotation.x, self.offset.x, -self.offset.y),
                Line::Horizontal(y) => (y, self.center_of_rotation.y, self.offset.y, self.offset.x),
            };

            // Already intersecting
            if (center_pos + offset_parallel).approx_eq(target) {
                return Some(0.);
            }

            // pos(t) = center + offset*cos(ωt) + perp_offset*sin(ωt)
            // Solve: pos(t) = target
            // center + offset*cos(ωt) + perp_offset*sin(ωt) = target
            // offset*cos(ωt) + perp_offset*sin(ωt) = target - center
            
            let separation = target - center_pos;
            
            let radius = self.offset.length();
            if radius.less_mag(separation) { return None }
            
            // Using the fact that offset = r*cos(θ) and perp_offset = r*sin(θ)
            // where r is radius and θ is initial angle
            // We can solve for the intersection time using atan2
            let initial_angle = offset_perp.atan2(offset_parallel);
            let target_angle = (separation / radius).acos();

            let t = (target_angle - initial_angle).normalize_angle() / self.angular_velocity;
            (t.greater(0.) && t.less_eq(max_time)).then_some(t)
        }

    }

    #[test]
    fn _manual_test() {
        let motion = Motion {
            center_of_rotation: Vec2::new(0.695020676, 0.),
            velocity: Vec2::new(-0.00475104246, 0.),
            offset: Vec2::new(-0.695016265, 0.130200937),
            angular_velocity: 0.170000225,
        };
        dbg!(motion.first_intersection(Line::Horizontal(0.), 1.0));
    }

}
