use std::ops::Range;

use crate::{Coord, Zone, Row};

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

    /// Get the column as an index. This is the column number as usize.
    #[inline]
    pub fn index(&self) -> usize {
        self.0 as usize
    }

    /// Unwrap the inner u8 value
    pub(crate) fn inner(self) -> u8 {
        self.0
    }
}

rowcol_fromint!(
    Col, Col::SIZE, "col", 
    u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize
);

impl Zone for Col {
    type All = Cols;

    #[inline]
    fn all() -> Self::All {
        Cols(0..Row::SIZE as u8)
    }

    type Indexes = Indexes;

    #[inline]
    fn indexes(&self) -> Self::Indexes {
        Indexes {
            range: 0..Col::SIZE as u8,
            col: *self, 
        }
    }
}


/// Iterator over a row.
pub struct Indexes {
    range: Range<u8>,
    col: Col, 
}

impl Indexes {
    #[inline]
    fn build_coord(&self, row: u8) -> Coord {
        Coord::new(row, self.col)
    }
}

zone_indexes_iter!(Indexes);

/// Iterator over all columns.
pub struct Cols(Range<u8>);

impl Cols {
    fn build_zone(c: u8) -> Col {
        Col(c)
    }
}

zone_all_iter!(Cols, Col);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn col_iter() {
        for c in 0..9 {
            let col = Col::new(c);
            let expected: Vec<_> = (0..9).map(|r| Coord::new(r, c)).collect();
            let result: Vec<_> = col.indexes().collect();
            assert_eq!(result, expected);
        }
    }
}