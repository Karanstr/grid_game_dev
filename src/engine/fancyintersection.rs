use super::*;
use roots::{find_root_brent, SimpleConvergency};

// Unify epsilon with the one found in collisions
const EPSILON:f32 = 1e-10;
//Figure out how far we can cut that iteration count down
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

    // Solves for time T when the motion intersects the given line
    // Returns the first time an intersection occurs, ignoring sign
    fn solve_line_intersection(&self, line: &Line, max_time: f32) -> Option<f32> {

        // Get the relevant components based on whether this is vertical or horizontal
        let (target, start, velocity, offset_x, offset_y) = match line {
            Line::Vertical(x) => (*x, self.start.x, self.velocity.x, self.offset.x, -self.offset.y),
            Line::Horizontal(y) => (*y, self.start.y, self.velocity.y, self.offset.y, self.offset.x),
        };

        // Create case for if both are negligible?

        // Handle pure rotation (no linear motion) separately
        if velocity.abs() < EPSILON { return self.solve_pure_rotation(line, max_time) }

        // Handle pure linear motion (no rotation) separately
        if self.angular_velocity.abs() < EPSILON {
            // For linear motion, the offset is constant (like at t=0)
            let effective_start = start + offset_x;  // Add the constant offset
            let t = (target - effective_start) / velocity;
            return if t.abs() <= max_time { Some(t) } else { None };
        }

        // Start with the linear intersection time as our best guess
        let t_linear = (target - start) / velocity;
        let period = 2.0 * std::f32::consts::PI / self.angular_velocity.abs();
        
        // Define the motion equation we're solving
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
            let t_start = (t_center - period/2.0).max(-max_time);  // Don't check beyond -max_time
            let t_end = (t_center + period/2.0).min(max_time);  // Don't check beyond max_time
            
            // If the entire period is out of bounds, skip it
            if t_start >= max_time || t_end <= -max_time {
                return None;
            }
            
            let f_start = f(t_start);
            let f_end = f(t_end);                                   
            let mut result = None;
            if f_start * f_end <= 0.0 {
                if let Ok(t) = find_root_brent(t_start, t_end, |t| f(t), &mut CONVERGENCY) {
                    if t.abs() <= max_time && t.abs() < current_best.abs() {
                        result = Some(t);
                    }
                }
            }
            result
        };

        // Check the period containing t_linear first
        if let Some(t) = check_period(period_index, closest_t) { closest_t = t }
        
        // Then check adjacent periods, moving outward until we either:
        // 1. Find a solution closer to 0 than what we have, or
        // 2. Determine that further periods would only yield solutions farther from 0
        let mut offset = 1;
        while closest_t.is_infinite() || (offset as f32 * period).abs() < closest_t.abs() {
            // If we're definitely beyond max_time in both directions, stop searching
            if (period_index + offset) as f32 * period > max_time && 
               (period_index - offset) as f32 * period < -max_time {
                break;
            }
            
            let mut found_closer = false;
            
            // Check positive offset if it might be within bounds
            if (period_index + offset) as f32 * period <= max_time {
                if let Some(t) = check_period(period_index + offset, closest_t) {
                    closest_t = t;
                    found_closer = true;
                }
            }
            
            // Check negative offset if it might be within bounds
            if (period_index - offset) as f32 * period >= -max_time {
                if let Some(t) = check_period(period_index - offset, closest_t) {
                    closest_t = t;
                    found_closer = true;
                }
            }
            
            if !closest_t.is_infinite() && found_closer { break }
            
            offset += 1;
        }
        
        if closest_t.is_infinite() { None } else { Some(closest_t) }
    }

    fn solve_pure_rotation(&self, line: &Line, max_time: f32) -> Option<f32> {
        //Can we unify these matches so we only match once instead of thrice?
        let (start, offset_x, offset_y) = match line {
            Line::Vertical(_) => (self.start.x, self.offset.x, self.offset.y),
            Line::Horizontal(_) => (self.start.y, self.offset.y, self.offset.x),
        };

        let r = (offset_x * offset_x + offset_y * offset_y).sqrt();
        if r < EPSILON { return None }

        let delta = match line {
            Line::Vertical(x) => x - start,
            Line::Horizontal(y) => y - start,
        };

        if delta.abs() > r { return None }

        let angle = match line {
            Line::Vertical(_) => (delta / r).acos() * delta.signum(),
            Line::Horizontal(_) => (delta / r).asin(),
        };

        let t = angle / self.angular_velocity;
        if t.abs() <= max_time { Some(t) } else { None }
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
        let wall_x = Line::Vertical(3.1);
        dbg!(motion.solve_line_intersection(&wall_x, 1.0));
    }
}
