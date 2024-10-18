use macroquad::prelude::*;
use sddag::NodeAddress;

mod sddag;

//Todo:
//Figure out what async does
#[macroquad::main("First Window")]
async fn main() {
    let mut tree  = sddag::SparseDAG1D::new(2);
    let mut root = NodeAddress::new(2, 0);
    tree.set_node_child(&mut root, 0, 0b000, 1);
    tree.set_node_child(&mut root, 0, 0b010, 1);


    let mut player = Object::new(
        Vec2::new(0.,0.), 
        Vec2::new(20.,20.),
    );

    let speed = 3.;

    loop {
        //Completely arbitrary, screen is cleared with or without
        clear_background(BLACK);

        player.velocity_as_wasd(speed);
        player.update_position();
        player.render(GREEN);

        next_frame().await
    }
}


struct Object {
    position : Vec2,
    velocity : Vec2,
    size : Vec2,
}

//Static methods
impl Object {
    fn new(pos:Vec2, dimensions:Vec2) -> Self {
        Self {
            position : pos,
            velocity : Vec2::new(0., 0.),
            size : dimensions
        }
    }
}

//Instance methods
impl Object {
    fn update_position(&mut self) {
        self.position += self.velocity;
        self.velocity.x = 0.;
        self.velocity.y = 0.;
    }

    fn velocity_as_wasd(&mut self, speed:f32) {
        if is_key_down(KeyCode::A) {
            self.velocity.x -= speed;
        }
        if is_key_down(KeyCode::D) {
            self.velocity.x += speed;
        }
        if is_key_down(KeyCode::W) {
            self.velocity.y -= speed;
        }
        if is_key_down(KeyCode::S) {
            self.velocity.y += speed;
        }

    }

    fn render(&self, color:Color) {
        draw_rectangle(self.position.x, self.position.y, self.size.x, self.size.y, color);
    }

}