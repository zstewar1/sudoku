use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::iter::FusedIterator;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::collections::indexed::FixedSizeIndex;
use crate::coordinates::{FixedSizeIndexable, ZoneContaining};
use crate::{Col, OutOfRange, Row, Sector, SectorCol, SectorRow, Zone};

/// Coordinates of a single cell on the Sudoku board.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Coord {
    /// Row (y).
    row: Row,
    /// Column (x).
    col: Col,
}

impl Coord {
    /// Construct a new coordinate. Since this is (row, col), note that it is (y, x).
    #[inline]
    pub fn new(row: Row, col: Col) -> Self {
        Coord { row, col }
    }

    /// Get the row of this coordinate (y).
    #[inline]
    pub fn row(&self) -> Row {
        self.row
    }

    /// Get the col of this coordinate (x).
    #[inline]
    pub fn col(&self) -> Col {
        self.col
    }

    /// Set the row of this coordinate (y).
    #[inline]
    pub fn set_row(&mut self, row: Row) {
        self.row = row.into();
    }

    /// Set the col of this coordinate (x).
    #[inline]
    pub fn set_col(&mut self, col: Col) {
        self.col = col.into();
    }

    /// Get the sector that this coordinate is in.
    #[inline]
    pub fn sector(&self) -> Sector {
        Sector::containing(*self)
    }

    /// Get the sector sub-row that this coordinate is in.
    #[inline]
    pub fn sector_row(&self) -> SectorRow {
        SectorRow::containing(*self)
    }

    /// Get the sector sub-column that this coordinate is in.
    #[inline]
    pub fn sector_col(&self) -> SectorCol {
        SectorCol::containing(*self)
    }

    /// Get all coordinates in the same row, column, and sector as this
    /// coordinate.
    pub fn neighbors(self) -> impl Iterator<Item = Coord> + DoubleEndedIterator + FusedIterator {
        self.row
            .coords()
            .chain(self.col.coords())
            .chain(
                self.sector()
                    .coords()
                    .filter(move |&other| !self.row.contains(other) && !self.col.contains(other)),
            )
            .filter(move |other| *other != self)
    }
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.row, self.col)
    }
}

impl<T, U> TryFrom<(T, U)> for Coord
where
    T: TryInto<Row> + Copy + fmt::Debug,
    U: TryInto<Col> + Copy + fmt::Debug,
{
    type Error = OutOfRange<(T, U)>;

    /// Converts an (y-row, x-col) pair to a Coordinate.
    fn try_from((row, col): (T, U)) -> Result<Self, Self::Error> {
        let r = row.try_into().map_err(|_| OutOfRange((row, col)))?;
        let c = col.try_into().map_err(|_| OutOfRange((row, col)))?;
        Ok(Coord::new(r, c))
    }
}

impl FixedSizeIndexable for Coord {
    type Item = Coord;

    /// Coords are a single cell.
    const NUM_ITEMS: usize = 1;

    fn get_at_index(&self, idx: usize) -> Self::Item {
        assert!(idx < Self::NUM_ITEMS, "index {} out of range", idx);
        *self
    }
}

impl IntoIterator for Coord {
    type Item = Coord;
    type IntoIter = std::iter::Once<Self>;

    fn into_iter(self) -> Self::IntoIter {
        std::iter::once(self)
    }
}

impl ZoneContaining for Coord {
    #[inline]
    fn containing_zone(coord: Coord) -> Self {
        coord.into()
    }
}

impl FixedSizeIndex for Coord {
    const NUM_INDEXES: usize = Row::SIZE * Col::SIZE;

    fn idx(&self) -> usize {
        self.row.idx() * Col::NUM_INDEXES + self.col.idx()
    }

    fn from_idx(idx: usize) -> Self {
        assert!(
            idx < Self::NUM_INDEXES,
            "flat index must be in range [0, {}), got {}",
            Self::NUM_INDEXES,
            idx
        );
        let row = Row::new((idx / Col::NUM_INDEXES) as u8);
        let col = Col::new((idx % Col::NUM_INDEXES) as u8);
        Coord { row, col }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coord_iter() {
        for r in 0..9 {
            for c in 0..9 {
                let coord = Coord::new(Row::new(r), Col::new(c));
                let result: Vec<_> = coord.coords().collect();
                assert_eq!(result, vec![coord]);
            }
        }
    }

    #[test]
    fn coords_iter() {
        let mut expected = Vec::with_capacity(81);
        for r in 0..9 {
            for c in 0..9 {
                expected.push(Coord::new(Row::new(r), Col::new(c)));
            }
        }
        let result: Vec<_> = Coord::values().collect();
        assert_eq!(result, expected);
        for (idx, val) in result.iter().enumerate() {
            assert_eq!(val.idx(), idx);
        }
    }

    #[test]
    fn coord_neighbors() {
        for r in 0..9 {
            for c in 0..9 {
                let mut expected = Vec::with_capacity(20);
                for cc in 0..9 {
                    if cc != c {
                        expected.push(Coord::new(Row::new(r), Col::new(cc)));
                    }
                }
                for rr in 0..9 {
                    if rr != r {
                        expected.push(Coord::new(Row::new(rr), Col::new(c)));
                    }
                }
                for rr in ((r - (r % 3))..).take(3) {
                    for cc in ((c - (c % 3))..).take(3) {
                        if rr != r && cc != c {
                            expected.push(Coord::new(Row::new(rr), Col::new(cc)));
                        }
                    }
                }
                let result: Vec<_> = Coord::new(Row::new(r), Col::new(c)).neighbors().collect();
                assert_eq!(result.len(), 20);
                assert_eq!(result, expected);
            }
        }
    }

    #[cfg(feature = "serde")]
    mod serde_tests {
        use super::*;

        #[test]
        fn serialize() {
            for r in 0..9 {
                for c in 0..9 {
                    let coord = Coord::new(Row::new(r), Col::new(c));
                    let ser = serde_json::to_string(&coord).expect("could not serialize");
                    let expected = format!(r#"{{"row":{},"col":{}}}"#, r, c);
                    assert_eq!(ser, expected);
                }
            }
        }

        #[test]
        fn deserialize() {
            for r in 0..9 {
                for c in 0..9 {
                    let expected = Coord::new(Row::new(r), Col::new(c));
                    let inp = format!(r#"{{"row": {}, "col": {}}}"#, r, c);
                    let de: Coord = serde_json::from_str(&inp).expect("could not deserialize");
                    assert_eq!(de, expected);
                }
            }
        }

        #[test]
        fn deserialize_out_of_range() {
            for r in (-1024i32..0).chain(9..1024) {
                for c in 0..9 {
                    let inp = format!(r#"{{"row": {}, "col": {}}}"#, r, c);
                    let de: Result<Coord, _> = serde_json::from_str(&inp);
                    assert!(de.is_err());
                }
            }

            for r in 0..9 {
                for c in (-1024i32..0).chain(9..1024) {
                    let inp = format!(r#"{{"row": {}, "col": {}}}"#, r, c);
                    let de: Result<Coord, _> = serde_json::from_str(&inp);
                    assert!(de.is_err());
                }
            }
        }

        #[test]
        fn deserialize_bad_type() {
            let de: Result<Coord, _> = serde_json::from_str(r#"{"row": null, "col": 3}"#);
            assert!(de.is_err());
            let de: Result<Coord, _> = serde_json::from_str(r#"{"row": "3", "col": 3}"#);
            assert!(de.is_err());
        }
    }
}
