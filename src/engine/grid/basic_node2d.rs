use super::graph::{Step, Path, Index, Node, GraphNode};

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

    fn as_usize(&self) -> usize {
        *self as usize
    }
}

#[derive(Debug, Clone)]
pub struct Cell {
    cell: [u32; 2],
    length: usize,
}
impl Cell {
    pub fn new(cell: [u32; 2], length: usize) -> Self {
        Self {
            cell,
            length,
        }
    }
    pub fn cell(&self) -> [u32; 2] { self.cell }
}
impl Default for Cell {
    fn default() -> Self {
        Self {
            cell: [0; 2],
            length: 0,
        }
    }
}
impl Path<2, Zorder2d> for Cell {
    type InternalStep = [u32; 2];
        fn to_internal(step: Zorder2d) -> Self::InternalStep {
            match step {
                Zorder2d::TopLeft     => [0, 0],
                Zorder2d::TopRight    => [1, 0],
                Zorder2d::BottomLeft  => [0, 1],
                Zorder2d::BottomRight => [1, 1],
            }
        }
    fn from_internal(step: Self::InternalStep) -> Zorder2d {
        match step {
            [0, 0] => Zorder2d::TopLeft,
            [1, 0] => Zorder2d::TopRight,
            [0, 1] => Zorder2d::BottomLeft,
            [1, 1] => Zorder2d::BottomRight,
            _ => unreachable!()
        }
    }
    fn push_internal(&mut self, step: Self::InternalStep) {
        self.cell[0] = (self.cell[0] << 1) | (step[0] & 1);
        self.cell[1] = (self.cell[1] << 1) | (step[1] & 1);
        self.length += 1;
    }
    fn pop_internal(&mut self) -> Option<Self::InternalStep> {
        if self.length == 0 { return None }
        let result = [self.cell[0] & 1, self.cell[1] & 1];
        self.cell[0] >>= 1;
        self.cell[1] >>= 1;
        self.length -= 1;
        Some(result)
    }

    fn len(&self) -> usize { self.length }
    fn step_at(&self, n: usize) -> Option<Zorder2d> {
        if self.length <= n { return None }
        let shift = self.length - 1 - n;
        let step = [(self.cell[0] >> shift) & 1, (self.cell[1] >> shift) & 1];
        Some(Self::from_internal(step))
    }
}

/// Interleaved yx yx yx yx...
#[derive(Debug, Clone)]
pub struct PackedCell {
    packed: u64,
    length: usize,
}
impl PackedCell {
    pub fn new(packed: u64, length: usize) -> Self {
        Self {
            packed,
            length,
        }
    }
    pub fn packed(&self) -> u64 { self.packed }
}
impl Default for PackedCell {
    fn default() -> Self {
        Self {
            packed: 0,
            length: 0,
        }
    }
}
impl Path<2, Zorder2d> for PackedCell {
    type InternalStep = u64;
    fn to_internal(step: Zorder2d) -> Self::InternalStep { step.as_usize() as u64 }
    fn from_internal(step: Self::InternalStep) -> Zorder2d {
        match step {
            0 => Zorder2d::TopLeft,
            1 => Zorder2d::TopRight,
            2 => Zorder2d::BottomLeft,
            3 => Zorder2d::BottomRight,
            _ => unreachable!()
        }
    }
    fn push_internal(&mut self, step: Self::InternalStep) {
        let shift = 62 - self.length * 2;  // start at 62
        self.packed |= (step & 0b11) << shift;
        self.length += 1;
    }
    fn pop_internal(&mut self) -> Option<Self::InternalStep> {
        if self.length == 0 { return None }
        self.length -= 1;
        let shift = 62 - self.length * 2;
        let result = (self.packed >> shift) & 0b11;
        self.packed &= !(0b11 << shift);
        Some(result)
    }

    fn len(&self) -> usize { self.length }
    fn step_at(&self, n: usize) -> Option<Zorder2d> {
        if self.length <= n { return None }
        let shift = 62 - n * 2;
        let step = (self.packed >> shift) & 0b11;
        Some(Self::from_internal(step))
    }
}

pub type BasicNode2d = [Index; 4];
impl Node<2> for BasicNode2d {
    const DEFAULT: Self = [0; 4];
    type NativeStep = Zorder2d;

    fn child(&self, child: impl Step<2>) -> Index { self[child.as_usize()] }

    fn with_child(&self, child: impl Step<2>, idx: Index) -> Self {
        let mut new = self.clone();
        new[child.as_usize()] = idx;
        new
    }

}
impl GraphNode<2> for BasicNode2d {}

