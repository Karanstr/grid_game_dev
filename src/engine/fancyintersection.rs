use super::*;
use roots::{find_root_brent, SimpleConvergency};
const EPSILON:f32 = f32::EPSILON;
// https://www.desmos.com/calculator/rtvp1esep0
const CONVERGENCY:SimpleConvergency<f32> = SimpleConvergency {
    eps: EPSILON,
    max_iter: 12,
};

#[derive(Debug, Clone, Copy)]
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
    fn solve_line_intersection(&self, line: Line, max_time: f32) -> Option<f32> {

        // Get the relevant components based on whether this is vertical or horizontal
        let (target, start, velocity, offset_x, offset_y) = match line {
            Line::Vertical(x) => (x, self.start.x, self.velocity.x, self.offset.x, -self.offset.y),
            Line::Horizontal(y) => (y, self.start.y, self.velocity.y, self.offset.y, self.offset.x),
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
        
        
        let mut closest_t = f32::INFINITY;
        
        // Helper function to check a specific time range and return the found time if better
        let check_range = |t_start: f32, t_end: f32, current_best: f32| -> Option<f32> {
            let f_start = f(t_start);
            let f_end = f(t_end);
            
            if f_start * f_end <= 0.0 {
                if let Ok(t) = find_root_brent(t_start, t_end, &f, &mut CONVERGENCY) {
                    if t.abs() <= max_time && t.abs() < current_best.abs() {
                        return Some(t);
                    }
                }
            }
            None
        };
        
        // Check initial period around t=0 (both positive and negative)
        let initial_pos_end = period.min(max_time);
        let initial_neg_start = (-period).max(-max_time);
        
        if let Some(t) = check_range(initial_neg_start, 0.0, closest_t) {
            closest_t = t;
        }
        if let Some(t) = check_range(0.0, initial_pos_end, closest_t) {
            if t.abs() < closest_t.abs() {
                closest_t = t;
            }
        }
        
        // Search additional periods if needed
        let mut period_num = 1.0;
        
        while period_num * period <= max_time {
            let mut found_closer = false;
            
            // Check positive period
            let pos_start = period_num * period;
            let pos_end = ((period_num + 1.0) * period).min(max_time);
            if let Some(t) = check_range(pos_start, pos_end, closest_t) {
                if t.abs() < closest_t.abs() {
                    closest_t = t;
                    found_closer = true;
                }
            }
            
            // Check negative period
            let neg_end = -period_num * period;
            let neg_start = -(period_num + 1.0) * period.max(-max_time);
            if let Some(t) = check_range(neg_start, neg_end, closest_t) {
                if t.abs() < closest_t.abs() {
                    closest_t = t;
                    found_closer = true;
                }
            }
            
            // If we didn't find a closer solution in this period, we can stop
            if !found_closer && closest_t != f32::INFINITY {
                break;
            }
            
            period_num += 1.0;
        }
        
        if closest_t.is_infinite() { None } else { Some(closest_t) }
    }

    fn solve_pure_rotation(&self, line: Line, max_time: f32) -> Option<f32> {
        //Can we unify these matches so we only match once instead of thrice?
        let (start, offset) = match line {
            Line::Vertical(_) => (self.start.x, self.offset),
            Line::Horizontal(_) => (self.start.y, self.offset),
        };

        let r = offset.length();
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
            start: Vec2::new(0., 0.),
            velocity: Vec2::new(5.0, 5.0),
            offset: Vec2::new(1., -1.),
            angular_velocity: 3.7,
        };
        dbg!(motion.solve_line_intersection(Line::Horizontal(2.), 1.0));

    }
}
