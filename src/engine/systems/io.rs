use super::*;
use macroquad::input::*;

pub mod input {
    use crate::engine::systems::editing::set_grid_cell;
    use super::*;
    pub fn handle_mouse_input(game_data: &mut GameData) {
        if is_mouse_button_down(MouseButton::Left) {
            let mouse_pos = game_data.camera.screen_to_world(Vec2::from(mouse_position()));
            set_grid_cell(ExternalPointer::new(InternalPointer::new(Index(1)), 0), mouse_pos, &mut game_data.entities, &mut game_data.graph);
        }
    }
}


pub mod output {
    use crate::GameData;

    use super::*;
    #[derive(new)]
    pub struct RenderingSystem;
    impl RenderingSystem {

    pub fn draw_all(game_data: &mut GameData) {
        let mut locations_to_draw = Vec::new();
        for (_, location) in game_data.entities.query_mut::<&mut Location>() {
            if game_data.camera.aabb.intersects(Bounds::aabb(location.position, location.pointer.height)) == BVec2::TRUE {
                locations_to_draw.push(location.clone());
            }
        }
        for location in locations_to_draw {
            Self::draw(game_data, &location);
        }
    }

    pub fn draw(game_data:&mut GameData, location:&Location) {
        let grid_length = Bounds::cell_length(location.pointer.height);
        let grid_top_left = location.position - grid_length / 2.;
        game_data.camera.outline_vec_rectangle(
            grid_top_left,
            grid_length,
            2.,
            WHITE
        );
        let object_top_left = location.position - grid_length / 2.;
        let leaves = game_data.graph.dfs_leaves(location.pointer);
        for leaf in leaves {
            let color = game_data.blocks.blocks[*leaf.pointer.pointer.index].color; 
            if 0 != *leaf.pointer.pointer.index {
                let cell_top_left = object_top_left + Bounds::top_left_corner(leaf.cell, leaf.pointer.height);
                game_data.camera.draw_vec_rectangle(
                cell_top_left,
                Bounds::cell_length(leaf.pointer.height),
                color
                );
                game_data.camera.outline_vec_rectangle(
                cell_top_left,
                Bounds::cell_length(leaf.pointer.height),
                2.,
                WHITE
                );
            }
        }
    }

    }



}


pub struct Camera { 
    pub aabb : AABB,
    scale_zoom: f32,
    zoom:f32,
    screen_percentage: f32,
}
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
        self.outline_bounds(self.aabb, 2., WHITE);
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
        draw_rectangle_lines(pos.x, pos.y, len.x, len.y, line_width, color);
    }
    
    pub fn draw_vec_circle(&self, position:Vec2, radius:f32, color:Color) {
        let pos = self.world_to_screen(position);
        draw_circle(pos.x, pos.y, radius * self.zoom(), color);
    }

    pub fn outline_vec_circle(&self, position:Vec2, radius:f32, line_width:f32, color:Color) {
        let pos = self.world_to_screen(position);
        draw_circle_lines(pos.x, pos.y, radius * self.zoom(), line_width, color);
    }

    pub fn draw_vec_line(&self, point1:Vec2, point2:Vec2, line_width:f32, color:Color) {
        let p1 = self.world_to_screen(point1);
        let p2 = self.world_to_screen(point2);
        draw_line(p1.x, p1.y, p2.x, p2.y, line_width, color);
    }

    pub fn outline_bounds(&self, bounds:AABB, line_width:f32, color:Color) {
        self.outline_vec_rectangle(bounds.min(), bounds.max() - bounds.min(), line_width, color);
    } 

}
