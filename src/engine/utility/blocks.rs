use macroquad::color::colors::*;
use macroquad::color::Color;
use derive_new::new;

#[derive(Debug, Clone, Copy, new)]
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