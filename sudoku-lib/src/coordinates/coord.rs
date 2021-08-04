use std::iter::FusedIterator;
use crate::{Col, Row, Sector, Zone};
use crate::collections::indexed::FixedSizeIndex;

/// Coordinates of a single cell on the Sudoku board.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Coord {
    /// Row (y).
    row: Row,
    /// Column (x).
    col: Col,
}

impl Coord {
    /// Construct a new coordinate. Since this is (row, col), note that it is (y, x).
    #[inline]
    pub fn new(row: impl Into<Row>, col: impl Into<Col>) -> Self {
        Coord {
            row: row.into(),
            col: col.into(),
        }
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
    pub fn set_row(&mut self, row: impl Into<Row>) {
        self.row = row.into();
    }

    /// Set the col of this coordinate (x).
    #[inline]
    pub fn set_col(&mut self, col: impl Into<Col>) {
        self.col = col.into();
    }

    /// Get the sector that this coordinate is in.
    #[inline]
    pub fn sector(&self) -> Sector {
        Sector::containing(*self)
    }

    /// Get all coordinates in the same row, column, and sector as this
    /// coordinate.
    pub fn neighbors(&self) -> impl Iterator<Item = Coord> + DoubleEndedIterator + FusedIterator {
        let copy = *self;
        self.row
            .coords()
            .chain(self.col.coords())
            .chain(
                self.sector().coords().filter(move |&other| {
                    !copy.row().contains(other) && !copy.col().contains(other)
                }),
            )
            .filter(move |other| other != &copy)
    }
}

impl<T: Into<Row>, U: Into<Col>> From<(T, U)> for Coord {
    /// Converts an (y-row, x-col) pair to a Coordinate.
    fn from((row, col): (T, U)) -> Self {
        Coord::new(row, col)
    }
}

impl Zone for Coord {
    /// Coords are a single cell.
    const SIZE: usize = 1;

    type Coords = std::iter::Once<Coord>;

    #[inline]
    fn coords(&self) -> Self::Coords {
        std::iter::once(*self)
    }

    #[inline]
    fn containing(coord: impl Into<Coord>) -> Self {
        coord.into()
    }

    #[inline]
    fn contains(&self, coord: impl Into<Coord>) -> bool {
        *self == coord.into()
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
        let row = (idx / Col::NUM_INDEXES).into();
        let col = (idx % Col::NUM_INDEXES).into();
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
                let coord = Coord::new(r, c);
                let result: Vec<_> = coord.coords().collect();
                assert_eq!(result, vec![coord]);
            }
        }
    }
}
