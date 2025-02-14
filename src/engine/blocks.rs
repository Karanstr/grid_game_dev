
#[derive(Debug, Clone, Copy)]
pub enum CollisionType {
    Solid,  // index 1 or 3
    Air,    // index 0 or 2
    Void,   // No block, unspecified behavior
}

use macroquad::color::*;
use super::grid::partition::CellData;

#[derive(Debug)]
struct Block {
    color : Color,
    collision_type : CollisionType
}

pub struct BlockPalette([Block; 4]);
impl Default for BlockPalette {
    fn default() -> Self {
        Self ( [
                Block {
                    color : BLACK,
                    collision_type : CollisionType::Air
                },
                Block {
                    color : GREEN,
                    collision_type : CollisionType::Solid
                },
                Block {
                    color : BLUE,
                    collision_type : CollisionType::Air
                },
                Block {
                    color : GRAY,
                    collision_type : CollisionType::Solid
                },
            ]
        )
    }
}
impl BlockPalette {
    pub fn index_type(&self, index : usize) -> CollisionType {
        self.0[index].collision_type
    }
    
    pub fn cell_type(&self, cell: Option<CellData>) -> CollisionType {
        match cell {
            None => CollisionType::Void,
            Some(cell) => self.index_type(*cell.pointer.pointer)
        }
    }

    pub fn color(&self, index : usize) -> Color {
        self.0[index].color
    }

    pub fn is_solid_cell(&self, cell: Option<CellData>) -> bool {
        matches!(self.cell_type(cell), CollisionType::Solid)
    }
    pub fn is_solid_index(&self, index : usize) -> bool {
        matches!(self.index_type(index), CollisionType::Solid)
    }
}