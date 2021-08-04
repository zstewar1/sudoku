use std::ops::Range;

use crate::{Col, Coord, Zone};
use crate::collections::indexed::FixedSizeIndex;

/// Uniquely identifies a single row on the sudoku board. That is all cells with
/// the same y coordinate.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Row(u8);

impl Row {
    /// Construt a row with the given index. Panic if out of bounds.
    #[inline]
    pub fn new(row: impl Into<Row>) -> Self {
        row.into()
    }

    /// Unwrap the inner u8 value
    pub(crate) fn inner(self) -> u8 {
        self.0
    }
}

rowcol_fromint!(
    Row,
    Row::SIZE,
    "row",
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

impl Zone for Row {
    type Coords = Coords;

    #[inline]
    fn coords(&self) -> Self::Coords {
        Coords {
            range: 0..Row::SIZE as u8,
            row: *self,
        }
    }

    #[inline]
    fn containing(coord: impl Into<Coord>) -> Self {
        coord.into().row()
    }

    #[inline]
    fn contains(&self, coord: impl Into<Coord>) -> bool {
        *self == Self::containing(coord)
    }
}

impl FixedSizeIndex for Row {
    // Number of rows is the size of a column.
    const NUM_INDEXES: usize = Col::SIZE;

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
    row: Row,
}

impl Coords {
    #[inline]
    fn build_coord(&self, col: u8) -> Coord {
        Coord::new(self.row, col)
    }
}

zone_coords_iter!(Coords);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_iter() {
        for r in 0..9 {
            let row = Row::new(r);
            let expected: Vec<_> = (0..9).map(|c| Coord::new(r, c)).collect();
            let result: Vec<_> = row.coords().collect();
            assert_eq!(result, expected);
        }
    }
}
