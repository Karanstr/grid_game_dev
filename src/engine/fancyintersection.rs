use super::*;
use roots::{find_root_brent, SimpleConvergency};
// https://www.desmos.com/calculator/rtvp1esep0

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

const BASE_CONVERGENCY: SimpleConvergency<f32> = SimpleConvergency {
    eps: FP_EPSILON,
    max_iter: 20,
};

#[derive(Debug, Clone, Copy)]
pub enum Line {
    Vertical(f32),   // x = value
    Horizontal(f32), // y = value
}

#[derive(Clone, Copy)]
pub struct Motion {
    pub center_of_rotation: Vec2,
    pub velocity: Vec2,
    pub offset: Vec2,
    pub angular_velocity: f32,
}
impl Motion {

    // Solves for time T when the motion intersects the given line
    // Returns the closest time an intersection occurs, ignoring sign
    pub fn solve_line_intersection(self, line: Line, max_time: f32) -> Option<f32> {
        match (self.velocity.is_zero(), self.angular_velocity.is_zero()) {
            (true, true) => None,
            (false, true) => self.solve_pure_linear(line, max_time),
            (true, false) => self.solve_pure_rotation(line, max_time),
            (false, false) => self.solve_linear_and_rotation(line, max_time),
        }
    }

    pub fn solve_linear_and_rotation(self, line: Line, max_time: f32) -> Option<f32> {
        // Get the relevant components based on whether this is vertical or horizontal
        let (target, center, velocity, offset_parallel, offset_perp) = match line {
            Line::Vertical(x) => (x, self.center_of_rotation.x, self.velocity.x, self.offset.x, -self.offset.y),
            Line::Horizontal(y) => (y, self.center_of_rotation.y, self.velocity.y, self.offset.y, self.offset.x),
        };

        // Define the motion equation we're solving
        let f = |t: f32| {
            let angle = t * self.angular_velocity;
            let (sin_t, cos_t) = angle.sin_cos();
            center + t*velocity + offset_parallel*cos_t + offset_perp*sin_t - target
        };
        
        // Helper function to check a specific time range and return the found time if better
        let check_range = |t_start: f32, t_end: f32| -> Option<f32> {
            let f_start = f(t_start);
            let f_end = f(t_end);
            
            if f_start * f_end <= 0.0 {
                let mut iter_tracker = IterationTracker { convergency: BASE_CONVERGENCY, iterations: 0 };
                if let Ok(t) = find_root_brent(t_start, t_end, &f, &mut iter_tracker) {
                    // if t <= max_time { // Only check if t is within max_time, no abs() needed
                        println!("Found in {} iterations", iter_tracker.iterations);
                        return Some(t);
                    // }
                }
            }
            None
        };

        // Calculate the earliest possible intersection time based on distance and maximum speed
        let radius = self.offset.length();
        
        // Calculate the minimum time needed to reach the target based on linear motion and rotation radius
        let min_time_needed = ((target-center).abs() - radius) / velocity.abs();
        
        // If we can't reach the target within max_time, return early
        if min_time_needed.greater(max_time) || min_time_needed.less(0.) {
            return None;
        }
        
        // Calculate time for a complete rotation
        let rotation_period = 2.0 * PI / self.angular_velocity.abs();
        
        // Start searching from the earliest possible time
        let mut search_start = min_time_needed;
        
        let mut it_count = 0;

        // Search each rotation period until we exceed max_time
        while search_start <= max_time {
            let search_end = search_start + rotation_period;
            if let Some(t) = check_range(search_start, search_end) {
                return Some(t.snap_zero()); // This will be the earliest intersection in this period
            }
            search_start = search_end;
            it_count += 1;
            if it_count == 100 { dbg!("Failed to reach target"); break }
        }
        
        // No intersection found within max_time
        None
    }

    pub fn solve_pure_linear(self, line: Line, max_time: f32) -> Option<f32> {
        // Get the relevant components based on whether this is vertical or horizontal
        let (target, center, velocity, offset) = match line {
            Line::Vertical(x) => (x, self.center_of_rotation.x, self.velocity.x, self.offset.x),
            Line::Horizontal(y) => (y, self.center_of_rotation.y, self.velocity.y, self.offset.y),
        };

        let effective_center = center + offset;
        let t = (target - effective_center) / velocity;
        // Only return positive times within max_time
        if t > 0.0 && t <= max_time { Some(t) } else { None }
    }

    pub fn solve_pure_rotation(&self, line: Line, max_time: f32) -> Option<f32> {
        // Extract the target value and relevant components based on line type
        let (target, center_pos, offset_parallel, offset_perp) = match line {
            Line::Vertical(x) => (x, self.center_of_rotation.x, self.offset.x, -self.offset.y),
            Line::Horizontal(y) => (y, self.center_of_rotation.y, self.offset.y, self.offset.x),
        };

        // If we're already on the line, return immediately
        if (center_pos + offset_parallel).approx_eq(target) {
            return Some(0.0);
        }

        // For pure rotation, the position follows:
        // pos(t) = center + offset*cos(ωt) + perp_offset*sin(ωt)
        // where ω is angular_velocity
        
        // Solve: pos(t) = target
        // center + offset*cos(ωt) + perp_offset*sin(ωt) = target
        // offset*cos(ωt) + perp_offset*sin(ωt) = target - center
        
        // Let A = target - center
        let a = target - center_pos;
        
        // If the radius of rotation is too small to ever reach the target, return None
        let radius = self.offset.length();
        if (a.abs() - radius).greater(0.0) {
            return None;
        }
        
        // Using the fact that offset = r*cos(θ) and perp_offset = r*sin(θ)
        // where r is radius and θ is initial angle
        // We can solve for the intersection time using atan2
        let initial_angle = offset_perp.atan2(offset_parallel);
        let target_angle = (a / radius).acos();
        
        // There are two possible angles that satisfy our equation: target_angle and -target_angle
        // We need to find which one is reachable first given our angular velocity
        let mut possible_angles = vec![
            target_angle - initial_angle,
            -target_angle - initial_angle
        ];
        
        // Normalize angles to be in the range [-π, π]
        possible_angles.iter_mut().for_each(|angle| {
            *angle = angle.normalize_angle();
            if *angle > PI {
                *angle -= 2. * PI;
            }
        });
        
        // Convert angles to times based on angular velocity and only keep positive times
        let possible_times: Vec<f32> = possible_angles.into_iter()
            .map(|angle| angle / self.angular_velocity)
            .filter(|&t| t > 0.0 && t <= max_time) // Only consider positive times
            .collect();
        
        // Return the smallest positive time
        possible_times.into_iter().min_by(|a, b| a.partial_cmp(b).unwrap())
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let motion = Motion {
            center_of_rotation: Vec2::new(0.695020676, 0.),
            velocity: Vec2::new(-0.00475104246, 0.),
            offset: Vec2::new(-0.695016265, 0.130200937),
            angular_velocity: 0.170000225,
        };
        dbg!(motion.solve_line_intersection(Line::Horizontal(0.), 1.0));

    }
}
