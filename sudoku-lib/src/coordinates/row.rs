use std::ops::Range;

use crate::{Col, Coord, Zone};

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

    /// Get the row as an index. This is the row number as usize.
    #[inline]
    pub fn index(self) -> usize {
        self.0 as usize
    }

    /// Unwrap the inner u8 value
    pub(crate) fn inner(self) -> u8 {
        self.0
    }
}

rowcol_fromint!(
    Row, Row::SIZE, "row", 
    u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize
);

impl Zone for Row {
    type All = Rows;

    #[inline]
    fn all() -> Self::All {
        Rows(0..Col::SIZE as u8)
    }

    type Indexes = Indexes;

    #[inline]
    fn indexes(&self) -> Self::Indexes {
        Indexes {
            range: 0..Row::SIZE as u8,
            row: *self, 
        }
    }
}

/// Iterator over a row.
pub struct Indexes {
    range: Range<u8>,
    row: Row, 
}

impl Indexes {
    #[inline]
    fn build_coord(&self, col: u8) -> Coord {
        Coord::new(self.row, col)
    }
}

zone_indexes_iter!(Indexes);

/// Iterator over all rows.
pub struct Rows(Range<u8>);

impl Rows {
    fn build_zone(r: u8) -> Row {
        Row(r)
    }
}

zone_all_iter!(Rows, Row);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn row_iter() {
        for r in 0..9 {
            let row = Row::new(r);
            let expected: Vec<_> = (0..9).map(|c| Coord::new(r, c)).collect();
            let result: Vec<_> = row.indexes().collect();
            assert_eq!(result, expected);
        }
    }
}