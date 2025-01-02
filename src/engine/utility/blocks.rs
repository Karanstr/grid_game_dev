use macroquad::color::colors::*;
use macroquad::color::Color;

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub color : Color,
}
pub struct BlockPalette {
    pub blocks : Vec<Block>
}
impl BlockPalette {
    pub fn new() -> Self {
        Self {
            blocks : vec![
                Block {
                    color : BLACK,
                },
                Block {
                    color : GREEN,
                },
                Block {
                    color : BLUE,
                },
                Block {
                    color : GRAY,
                },
            ]
        }
    }
}