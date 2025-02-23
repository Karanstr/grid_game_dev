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

    #[derive(Debug, Clone, Copy, new)]
    pub struct Motion {
        pub center_of_rotation: Vec2,
        pub offset: Vec2,
        pub velocity: Vec2,
        pub rotational_velocity: f32,
        // Center of revolution is origin
        pub revolutionary_velocity: f32,
    }
    impl Motion {

        pub fn at_tick(self, ticks: f32) -> Vec2 {
            // https://www.desmos.com/calculator/frrsklqt5y
            let rotation = Vec2::from_angle(ticks * self.rotational_velocity);
            let revolution = Vec2::from_angle(ticks * self.revolutionary_velocity);
            let orbit_point = self.offset.rotate(rotation) + ticks*self.velocity;
            (orbit_point + self.center_of_rotation).rotate(revolution)
        }

        // pub fn first_intersection(self, line: Line, max_time: f32) -> Option<f32> {
        //     // match (self.velocity.is_zero(), self.angular_velocity.is_zero()) {
        //     //     (true, true) => None,
        //     //     (false, true) => self.solve_pure_linear(line, max_time),
        //     //     // (true, false) => self.solve_pure_rotation(line, max_time),
        //     //     // (false, false) => self.solve_linear_and_rotation(line, max_time),
        //     //     _ => self.solve_linear_and_rotation(line, max_time),
        //     // }
        //     self.solve_linear_and_rotation(line, max_time)
            
        // }

        // fn solve_linear_and_rotation(self, line: Line, max_time: f32) -> Option<f32> {
        fn solve_linear_and_rotation(self, vertical_line: f32, max_time: f32) -> Option<f32> {
        
            // let (target_offset, velocity, offset_parallel, offset_perp) = match line {
            //     Line::Vertical(x) => (self.center_of_rotation.x - x, self.velocity.x, self.offset.x, -self.offset.y),
            //     Line::Horizontal(y) => (self.center_of_rotation.y - y, self.velocity.y, self.offset.y, self.offset.x),
            // };

            let f = |t: f32| vertical_line - self.at_tick(t).x;
            
            let check_range = |t_start: f32, t_end: f32| -> Option<f32> {
                let mut iter_tracker = IterationTracker::default();
                if let Ok(t) = find_root_brent(t_start, t_end, &f, &mut iter_tracker) {
                    println!("Found in {} iterations", iter_tracker.iterations);
                    return Some(t)
                }
                None
            };

            // // Calculate the minimum time needed to potentially reach the target based on linear motion and rotation radius
            // let radius = self.offset.length();
            // let min_time_needed = if radius.greater_eq_mag(target_offset) { 0. } else {
            //     (target_offset.abs() - radius) / velocity
            // };
            // if min_time_needed.greater(max_time) { return None }
            
            // let rotation_period = 2. * PI / self.angular_velocity.abs();
            
            // check_range(min_time_needed, (min_time_needed + rotation_period).min(max_time))

            check_range(0., max_time)
        }

/*
        fn solve_pure_linear(self, line: Line, max_time: f32) -> Option<f32> {
            let (target, center, velocity, offset) = match line {
                Line::Vertical(x) => (x, self.center_of_rotation.x, self.velocity.x, self.offset.x),
                Line::Horizontal(y) => (y, self.center_of_rotation.y, self.velocity.y, self.offset.y),
            };
            let point = center + offset;
            let t = (target - point) / velocity;
            (t.greater(0.) && t.less_eq(max_time)).then_some(t)
        }

        fn solve_pure_rotation(self, line: Line, max_time: f32) -> Option<f32> { todo!() }
        */
    }

    #[test]
    fn _manual_test() {
        let motion = Motion {
            center_of_rotation: Vec2::new(0.4, 0.),
            offset : Vec2::new(0.5, 0.5),
            velocity: Vec2::new(0., -2.),
            rotational_velocity: 0.2,
            revolutionary_velocity: 0.3,
        };
        dbg!(motion.at_tick(1.0));
        dbg!(motion.solve_linear_and_rotation(1., 1.));
    }

}
