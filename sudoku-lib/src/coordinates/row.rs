use crate::collections::indexed::FixedSizeIndex;
use crate::coordinates::{ZoneContaining, FixedSizeIndexable};
use crate::{Col, Coord, Zone};

/// Uniquely identifies a single row on the sudoku board. That is all cells with
/// the same y coordinate.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Row(u8);

impl Row {
    /// Construt a row with the given index. Panic if out of bounds.
    #[inline]
    pub fn new(row: impl Into<Row>) -> Self {
        row.into()
    }

    /// Unwrap the inner u8 value
    #[inline]
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

impl FixedSizeIndexable for Row {
    type Item = Coord;

    const NUM_ITEMS: usize = 9;

    #[inline]
    fn get_at_index(&self, idx: usize) -> Self::Item {
        Coord::new(*self, idx)
    }
}

impl ZoneContaining for Row {
    #[inline]
    fn containing_zone(coord: impl Into<Coord>) -> Self {
        coord.into().row()
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
