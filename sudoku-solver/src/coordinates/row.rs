use std::convert::TryInto;
use std::fmt;
use std::iter::FusedIterator;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::collections::indexed::FixedSizeIndex;
use crate::coordinates::{FixedSizeIndexable, ZoneContaining};
use crate::{Col, Coord, Sector, SectorRow, Zone};

/// Uniquely identifies a single row on the sudoku board. That is all cells with
/// the same y coordinate.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[repr(transparent)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "u8"),
    serde(into = "u8")
)]
pub struct Row(u8);

impl Row {
    /// Width of a row as a number of columns.
    pub const WIDTH: u8 = 9;

    /// Construt a row with the given index. Panic if out of bounds.
    #[inline]
    pub fn new(val: u8) -> Self {
        assert!((0..Self::NUM_INDEXES as u8).contains(&val));
        Self(val)
    }

    /// Unwrap the inner u8 value
    #[inline]
    pub fn inner(self) -> u8 {
        self.0
    }

    /// Iterator over `SectorRow` in this `Row`.
    pub(crate) fn sector_rows(
        self,
    ) -> impl Iterator<Item = SectorRow> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        (0..Sector::SECTORS_ACROSS)
            .map(move |c| SectorRow::containing_zone(Coord::new(self, Col::new(c * Sector::WIDTH))))
    }

    /// Base-row for sectors that contain this row.
    pub(crate) fn sector_base(self) -> Self {
        Row(self.0 - self.0 % Sector::HEIGHT)
    }
}

impl fmt::Display for Row {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "row {}", self.0)
    }
}

rowcol_fromint!(
    Row,
    Row::WIDTH,
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

    const NUM_ITEMS: usize = Self::WIDTH as usize;

    #[inline]
    fn get_at_index(&self, idx: usize) -> Self::Item {
        Coord::new(*self, idx.try_into().expect("index out of range"))
    }
}

fixed_size_indexable_into_iter!(Row);

impl ZoneContaining for Row {
    #[inline]
    fn containing_zone(coord: Coord) -> Self {
        coord.row()
    }
}

impl FixedSizeIndex for Row {
    // Number of rows is the size of a column.
    const NUM_INDEXES: usize = Col::SIZE;

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
    fn row_iter() {
        for r in 0..9 {
            let row = Row::new(r);
            let expected: Vec<_> = (0..9).map(|c| Coord::new(Row(r), Col::new(c))).collect();
            let result: Vec<_> = row.coords().collect();
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn rows_iter() {
        let mut expected = Vec::with_capacity(9);
        for r in 0..9 {
            expected.push(Row::new(r));
        }
        let result: Vec<_> = Row::values().collect();
        assert_eq!(result, expected);
        assert_sorted!(result);
        for (idx, val) in result.iter().enumerate() {
            assert_eq!(val.idx(), idx);
        }
    }

    #[cfg(feature = "serde")]
    mod serde_tests {
        use super::*;

        #[test]
        fn serialize() {
            for r in 0..9 {
                let row = Row::new(r);
                let ser = serde_json::to_string(&row).expect("could not serialize");
                assert_eq!(ser, r.to_string());
            }
        }

        #[test]
        fn deserialize() {
            for r in 0..9 {
                let expected = Row::new(r);
                let de: Row = serde_json::from_str(&r.to_string()).expect("could not deserialize");
                assert_eq!(de, expected);
            }
        }

        #[test]
        fn deserialize_out_of_range() {
            for r in (-1024i32..0).chain(9..1024) {
                let de: Result<Row, _> = serde_json::from_str(&r.to_string());
                assert!(de.is_err());
            }
        }
    }
}
