use super::*;

#[derive(Debug, Clone, Copy)]
pub enum CollisionType {
    Solid,  // index 1 or 3
    Air,    // index 0 or 2
    Void,   // None
}


#[derive(Debug, new)]
pub struct Block {
    pub color : Color,
    pub collision_type : CollisionType
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
        }
    }
    pub fn index_type(&self, index : usize) -> CollisionType {
        self.blocks[index].collision_type
    }
    pub fn cell_type(&self, cell: Option<CellData>) -> CollisionType {
        match cell {
            None => CollisionType::Void,
            Some(cell) => self.index_type(*cell.pointer.pointer)
        }
    }
    pub fn is_solid_cell(&self, cell: Option<CellData>) -> bool {
        matches!(self.cell_type(cell), CollisionType::Solid)
    }
    pub fn is_solid_index(&self, index : usize) -> bool {
        matches!(self.index_type(index), CollisionType::Solid)
    }
}