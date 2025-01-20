use super::*;

pub mod input {
    use super::*;
    //I don't like this, use set_grid_cell directly?
    pub fn handle_mouse_input<T:GraphNode>(camera:&Camera, graph:&mut SparseDirectedGraph<T>, location:&mut Location, color: usize, height: u32) {
        if is_mouse_button_down(MouseButton::Left) {
            let mouse_pos = camera.screen_to_world(Vec2::from(mouse_position()));
            let new_pointer = ExternalPointer::new(Index(color), height);
            if let Some(pointer) = set_grid_cell(new_pointer, mouse_pos, *location, graph) {
                location.pointer = pointer;
            }
        }
    }

    use editing::*;
    mod editing {
        use super::*;
        pub fn set_grid_cell<T : GraphNode + std::hash::Hash + Eq>(to:ExternalPointer, world_point:Vec2, location:Location, graph:&mut SparseDirectedGraph<T>) -> Option<ExternalPointer> {
            let height = to.height;
            if height <= location.pointer.height {
                let cell = gate::point_to_cells(location, height, world_point)[0];
                if let Some(cell) = cell {
                    let path = ZorderPath::from_cell(cell, location.pointer.height - height);
                    if let Ok(pointer) = graph.set_node(location.pointer, &path.steps(), to.pointer) {
                        return Some(pointer)
                    } else {dbg!("Write failure. That's really bad.");}
                }
            }
            None
        }

        /*
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
        }*/
    }

}

pub mod output {
    use super::*;

    pub mod render {
        use super::*;
        
        pub fn draw_all<T:GraphNode>(camera:&Camera, graph:&SparseDirectedGraph<T>, entities:&EntityPool, blocks:&BlockPalette, outline:bool) {
            for entity in entities.entities.iter() {
                let location = entity.location;
                if camera.aabb.intersects(bounds::aabb(location.position, location.pointer.height)) == BVec2::TRUE {
                    draw(camera, graph, entity, blocks, outline);
                }
            }
        }
    
        pub fn draw<T:GraphNode>(camera:&Camera, graph:&SparseDirectedGraph<T>, entity:&Entity, blocks:&BlockPalette, outline:bool) {
            let grid_length = bounds::cell_length(entity.location.pointer.height);
            let grid_top_left = entity.location.position - grid_length / 2.;
            let grid_center = entity.location.position;
            let leaves = graph.dfs_leave_cells(entity.location.pointer);
            let angle = entity.forward;
            for leaf in leaves {
                if 0 != *leaf.pointer.pointer {
                    let color = blocks.blocks[*leaf.pointer.pointer].color; 
                    let length = bounds::cell_length(leaf.pointer.height);
                    let center = grid_top_left + bounds::top_left_corner(leaf.cell, leaf.pointer.height) + length / 2.;
                    let mut origin_center = center - grid_center;
                    origin_center.x = origin_center.x * angle.x - origin_center.y * angle.y;
                    origin_center.y = origin_center.x * angle.y + origin_center.y * angle.x;
                    let cell_center = grid_center + origin_center;
                    camera.draw_rotated_rectangle(cell_center, length, entity.rotation, color);
                }
                
            }
        }
    
    }

}

use macroquad::shapes::{draw_rectangle, draw_rectangle_lines, draw_line, draw_triangle};
use macroquad::miniquad::window::screen_size;
pub struct Camera { 
    pub aabb : AABB,
    scale_zoom: f32,
    zoom:f32,
    screen_percentage: f32,
}
#[allow(dead_code)]
impl Camera {
    pub fn new(aabb:AABB, screen_percentage:f32) -> Self {
        let scale_zoom = (Vec2::from(screen_size()) * screen_percentage).min_element() / (2. * aabb.radius().min_element());
        Self { 
            aabb, 
            scale_zoom,
            zoom: 1.,
            screen_percentage
        }
    }

    pub fn update(&mut self, new_position:Vec2, smoothing:f32) {
        self.lerp_position(new_position, smoothing);
        self.scale_zoom = (Vec2::from(screen_size())*self.screen_percentage).min_element() / (2. * self.aabb.radius().min_element());
    }

    pub fn change_zoom(&mut self, zoom:f32) { self.zoom *= zoom }

    pub fn change_screen_percentage(&mut self, screen_percentage:f32) {
        self.screen_percentage = screen_percentage;
        self.update(self.aabb.center(), 0.);
    }

    fn zoom(&self) -> f32 { self.zoom * self.scale_zoom }

    pub fn show_view(&self) {
        self.outline_bounds(self.aabb, 0.05, WHITE);
    }

    fn lerp_position(&mut self, position:Vec2, smoothing:f32) {
        self.aabb.move_to(self.aabb.center().lerp(position, smoothing));
    }
 
}
impl Camera {
    fn global_offset(&self) -> Vec2 {
        self.aabb.center() - Vec2::from(screen_size()) / 2. / self.zoom()
    }

    pub fn world_to_screen(&self, world_position:Vec2) -> Vec2 {
       (world_position - self.global_offset()) * self.zoom()
    }

    pub fn screen_to_world(&self, screen_position:Vec2) -> Vec2 {
        screen_position / self.zoom() + self.global_offset()
    }
}
impl Camera {
    pub fn draw_vec_rectangle(&self, position:Vec2, length:Vec2, color:Color) {
        let pos = self.world_to_screen(position);
        let len = length * self.zoom();
        draw_rectangle(pos.x, pos.y, len.x, len.y, color);
    }

    pub fn outline_vec_rectangle(&self, position:Vec2, length:Vec2, line_width:f32, color:Color) {
        let pos = self.world_to_screen(position);
        let len = length * self.zoom();
        draw_rectangle_lines(pos.x, pos.y, len.x, len.y, line_width*self.zoom(), color);
    }
    
    #[allow(dead_code)]
    pub fn draw_vec_line(&self, point1:Vec2, point2:Vec2, line_width:f32, color:Color) {
        let p1 = self.world_to_screen(point1);
        let p2 = self.world_to_screen(point2);
        draw_line(p1.x, p1.y, p2.x, p2.y, line_width*self.zoom(), color);
    }

    pub fn outline_bounds(&self, bounds:AABB, line_width:f32, color:Color) {
        self.outline_vec_rectangle(bounds.min(), bounds.max() - bounds.min(), line_width, color);
    } 

    pub fn draw_rotated_rectangle(&self, center: Vec2, length: Vec2, angle: f32, color: Color) {
        let pos = self.world_to_screen(center);
        let len = length * self.zoom();
        let half_len = len / 2.0;
        
        // Calculate the four corners of the rotated rectangle
        let cos_angle = angle.cos();
        let sin_angle = angle.sin();
        
        let corners = [
            Vec2::new(-half_len.x, -half_len.y),
            Vec2::new(half_len.x, -half_len.y),
            Vec2::new(half_len.x, half_len.y),
            Vec2::new(-half_len.x, half_len.y),
        ].map(|corner| {
            Vec2::new(
                corner.x * cos_angle - corner.y * sin_angle + pos.x,
                corner.x * sin_angle + corner.y * cos_angle + pos.y
            )
        });

        draw_triangle(
            corners[0],
            corners[1],
            corners[2],
            color
        );
        draw_triangle(
            corners[0],
            corners[2],
            corners[3],
            color
        );
    }

}
