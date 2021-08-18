use std::convert::TryInto;
use std::fmt;
use std::iter::FusedIterator;

use crate::collections::indexed::FixedSizeIndex;
use crate::coordinates::{FixedSizeIndexable, ZoneContaining};
use crate::{Coord, Row, Sector, SectorCol, Zone};

/// Uniquely identifies a single column on the sudoku board. That is all cells
/// with the same x coordinate.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Col(u8);

impl Col {
    /// Construt a column with the given index. Panic if out of bounds.
    #[inline]
    pub fn new(val: u8) -> Self {
        assert!((0..Self::NUM_INDEXES as u8).contains(&val));
        Self(val)
    }

    /// Unwrap the inner u8 value
    #[inline]
    pub(crate) fn inner(self) -> u8 {
        self.0
    }

    pub(crate) fn sector_cols(
        self,
    ) -> impl Iterator<Item = SectorCol> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        (0..Sector::SECTORS_DOWN).map(move |r| {
            SectorCol::containing_zone(Coord::new(Row::new(r * Sector::HEIGHT), self))
        })
    }
}

impl fmt::Display for Col {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "column {}", self.0)
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

impl FixedSizeIndexable for Col {
    type Item = Coord;

    const NUM_ITEMS: usize = 9;

    #[inline]
    fn get_at_index(&self, idx: usize) -> Self::Item {
        Coord::new(idx.try_into().expect("index out of range"), *self)
    }
}

fixed_size_indexable_into_iter!(Col);

impl ZoneContaining for Col {
    #[inline]
    fn containing_zone(coord: Coord) -> Self {
        coord.col()
    }
}

impl FixedSizeIndex for Col {
    // Number of columns is the size of a row.
    const NUM_INDEXES: usize = Row::SIZE;

    fn idx(&self) -> usize {
        self.0 as usize
    }

    fn from_idx(idx: usize) -> Self {
        idx.try_into().expect("index out of range")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn col_iter() {
        for c in 0..9 {
            let col = Col::new(c);
            let expected: Vec<_> = (0..9).map(|r| Coord::new(Row::new(r), Col(c))).collect();
            let result: Vec<_> = col.coords().collect();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn cols_iter() {
        let mut expected = Vec::with_capacity(9);
        for c in 0..9 {
            expected.push(Col::new(c));
        }
        let result: Vec<_> = Col::values().collect();
        assert_eq!(result, expected);
        for (idx, val) in result.iter().enumerate() {
            assert_eq!(val.idx(), idx);
        }
    }
}
