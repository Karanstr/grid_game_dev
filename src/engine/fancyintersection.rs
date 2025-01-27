use super::*;
use roots::{find_root_brent, SimpleConvergency};

const EPSILON:f32 = 1e-10;
const CONVERGENCY:SimpleConvergency<f32> = SimpleConvergency {
    eps: EPSILON,
    max_iter: 50,
};

#[derive(Debug)]
pub enum Line {
    Vertical(f32),   // x = value
    Horizontal(f32), // y = value
}

pub struct Motion {
    pub start: Vec2,
    pub velocity: Vec2,
    pub offset: Vec2,
    pub angular_velocity: f32,
}

impl Motion {
    /// Solves for time T when the motion intersects the given line
    /// Returns the first time an intersection occurs, ignoring sign
    pub fn solve_intersection(&self, line: &Line) -> Option<f32> {
        match line {
            Line::Vertical(x) => self.solve_line_intersection(*x, true),
            Line::Horizontal(y) => self.solve_line_intersection(*y, false),
        }
    }

    fn solve_line_intersection(&self, target: f32, is_vertical: bool) -> Option<f32> {
        // Get the relevant components based on whether this is vertical or horizontal
        let (start, velocity, offset_x, offset_y) = if is_vertical {
            (self.start.x, self.velocity.x, self.offset.x, -self.offset.y)
        } else {
            (self.start.y, self.velocity.y, self.offset.y, self.offset.x)
        };

        // Handle pure rotation (no linear motion) separately
        if velocity.abs() < EPSILON {
            return self.solve_pure_rotation(target, is_vertical);
        }

        // Handle pure linear motion (no rotation) separately
        if self.angular_velocity.abs() < EPSILON {
            // For linear motion, the offset is constant (like at t=0)
            let effective_start = start + offset_x;  // Add the constant offset
            let t = (target - effective_start) / velocity;
            return Some(t);
        }

        // Start with the linear intersection time as our best guess
        let t_linear = (target - start) / velocity;
        let period = 2.0 * std::f32::consts::PI / self.angular_velocity.abs();
        
        // Define the function we're finding roots for
        let f = |t: f32| {
            let angle = t * self.angular_velocity;
            let (sin_t, cos_t) = angle.sin_cos();
            start + t*velocity + offset_x*cos_t + offset_y*sin_t - target
        };
        
        // First check the period containing t_linear
        let period_index = (t_linear / period).floor() as i32;
        let mut closest_t = f32::INFINITY;
        
        // Helper function to check a specific period and return the found time if better
        let check_period = |period_index: i32, current_best: f32| -> Option<f32> {
            let t_center = period_index as f32 * period;
            let t_start = t_center - period/2.0;
            let t_end = t_center + period/2.0;
            
            let f_start = f(t_start);
            let f_end = f(t_end);
            let mut result = None;
            if f_start * f_end <= 0.0 {
                if let Ok(t) = find_root_brent(t_start, t_end, |t| f(t), &mut CONVERGENCY) {
                    if t.abs() < current_best.abs() { result = Some(t); }
                }
            }
            result
        };

        // Check the period containing t_linear first
        if let Some(t) = check_period(period_index, closest_t) {
            closest_t = t;
        }
        
        // Then check adjacent periods, moving outward until we either:
        // 1. Find a solution closer to 0 than what we have, or
        // 2. Determine that further periods would only yield solutions farther from 0
        let mut offset = 1;
        while closest_t.is_infinite() || (offset as f32 * period).abs() < closest_t.abs() {
            let mut found_closer = false;
            
            // Check positive offset
            if let Some(t) = check_period(period_index + offset, closest_t) {
                closest_t = t;
                found_closer = true;
            }
            
            // Check negative offset
            if let Some(t) = check_period(period_index - offset, closest_t) {
                closest_t = t;
                found_closer = true;
            }
            
            // If we found a solution and the next period would be further away,
            // we can stop searching
            if !closest_t.is_infinite() && found_closer { break }
            
            offset += 1;
        }
        
        if closest_t.is_infinite() { None } else { Some(closest_t) }
    }

    fn solve_pure_rotation(&self, target: f32, is_vertical: bool) -> Option<f32> {
        let (start, offset_x, offset_y) = if is_vertical {
            (self.start.x, self.offset.x, self.offset.y)
        } else {
            (self.start.y, self.offset.x, self.offset.y)
        };

        let delta = target - start;
        let r = (offset_x * offset_x + offset_y * offset_y).sqrt();
        
        if delta.abs() > r {
            return None;
        }

        let angle = if is_vertical {
            (delta / r).acos()
        } else {
            (delta / r).asin()
        };

        Some(angle / self.angular_velocity)
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let motion = Motion {
            start: Vec2::new(2.19, 6.6),
            velocity: Vec2::new(5.0, 5.0),
            offset: Vec2::ONE,
            angular_velocity: 0.,
        };
        let wall_x = 6.69;
        dbg!(motion.solve_intersection(&Line::Vertical(wall_x)));
    }
}
