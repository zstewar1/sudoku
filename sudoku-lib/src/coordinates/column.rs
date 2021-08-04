use std::ops::Range;

use crate::{Coord, Row, Zone};
use crate::collections::indexed::FixedSizeIndex;

/// Uniquely identifies a single column on the sudoku board. That is all cells
/// with the same x coordinate.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Col(u8);

impl Col {
    /// Construt a column with the given index. Panic if out of bounds.
    #[inline]
    pub fn new(col: impl Into<Col>) -> Self {
        col.into()
    }

    /// Unwrap the inner u8 value
    pub(crate) fn inner(self) -> u8 {
        self.0
    }
}

rowcol_fromint!(
    Col,
    Col::SIZE,
    "col",
    u8,
    i8,
    u16,
    i16,
    u32,
    i32,
    u64,
    i64,
    u128,
    i128,
    usize,
    isize
);

impl Zone for Col {
    type Coords = Coords;

    #[inline]
    fn coords(&self) -> Self::Coords {
        Coords {
            range: 0..Col::SIZE as u8,
            col: *self,
        }
    }

    #[inline]
    fn containing(coord: impl Into<Coord>) -> Self {
        coord.into().col()
    }

    #[inline]
    fn contains(&self, coord: impl Into<Coord>) -> bool {
        *self == Self::containing(coord)
    }
}

impl FixedSizeIndex for Col {
    // Number of columns is the size of a row.
    const NUM_INDEXES: usize = Row::SIZE;

    fn idx(&self) -> usize {
        self.0 as usize
    }

    fn from_idx(idx: usize) -> Self {
        idx.into()
    }
}

/// Iterator over a row.
pub struct Coords {
    range: Range<u8>,
    col: Col,
}

impl Coords {
    #[inline]
    fn build_coord(&self, row: u8) -> Coord {
        Coord::new(row, self.col)
    }
}

zone_coords_iter!(Coords);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn col_iter() {
        for c in 0..9 {
            let col = Col::new(c);
            let expected: Vec<_> = (0..9).map(|r| Coord::new(r, c)).collect();
            let result: Vec<_> = col.coords().collect();
            assert_eq!(result, expected);
        }
    }
}
