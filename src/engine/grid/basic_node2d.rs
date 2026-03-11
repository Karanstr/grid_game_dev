use super::graph::{Index, GraphNode, Node, Step};

// Add compressed to-from Zorder
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum Zorder2d {
  TopLeft,
  TopRight,
  BottomLeft,
  BottomRight,
}
impl Step<2> for Zorder2d {
  fn all() -> &'static [Zorder2d] {
    &[
      Self::TopLeft,
      Self::TopRight,
      Self::BottomLeft,
      Self::BottomRight,
    ]
  }

  fn new(subcell: [u32; 2]) -> Option<Self> {
    Some( match subcell {
      [0, 0] => Self::TopLeft,
      [1, 0] => Self::TopRight,
      [0, 1] => Self::BottomLeft,
      [1, 1] => Self::BottomRight,
      _ => None?
    })
  }

  fn subcell(&self) -> [u32; 2] {
    match self {
      Self::TopLeft     => [0, 0],
      Self::TopRight    => [1, 0],
      Self::BottomLeft  => [0, 1],
      Self::BottomRight => [1, 1],
    }
  }

  fn extract_last_step(cell: &mut [u32; 2]) -> Option<Self> {
    let subcell = [cell[0] & 0b1, cell[1] & 0b1];
    cell[0] >>= 1; cell[1] >>= 1;
    Self::new(subcell)
  }

  fn path_to_cell(path: &[Self]) -> [u32; 2] {
    let mut cell = [0; 2];
    for &step in path {
      let delta = step.subcell();
      cell[0] = cell[0] << 1 | delta[0];
      cell[1] = cell[1] << 1 | delta[1];
    }
    cell
  }

}

pub type BasicNode2d = [Index; 4];
impl Node<2> for BasicNode2d {
  type Children = Zorder2d;
  const DEFAULT: Self = [0; 4];

  fn get(&self, child: Self::Children) -> Index { self[child as usize] }

  fn with_child(&self, child: Self::Children, idx: Index) -> Self {
    let mut new = self.clone();
    new[child as usize] = idx;
    new
  }

}
impl GraphNode<2> for BasicNode2d {}


