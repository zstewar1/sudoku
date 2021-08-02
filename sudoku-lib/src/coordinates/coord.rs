use std::ops::Range;

use crate::{Board, Col, Row, Sector, Zone};

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

    /// Convert the row/column index into a linear board index as a usize to
    /// allow indexing the actual slice holding the cells.
    /// 
    /// This implements row-major indexing, where each row is in contiguous space (can be 
    /// pulled out as a slice) while columns are spread across rows. A row slice could be
    /// indexed by column.
    #[inline]
    pub(crate) fn flat_index(&self) -> usize {
        self.row.index() * Row::SIZE as usize + self.col.index()
    }

    /// Converts a flat index to a coordinate.
    pub(crate) fn from_flat_index(idx: usize) -> Self {
        assert!(idx < Board::SIZE, "flat index must be in range [0, {}), got {}", Board::SIZE, idx);
        let row = (idx / Row::SIZE as usize).into();
        let col = (idx % Row::SIZE as usize).into();
        Coord { row, col }
    }

    /// Get the sector that this coordinate is in.
    #[inline]
    pub fn sector(&self) -> Sector {
        Sector::containing(*self)
    }

    /// Get all coordinates in the same row, column, and sector as this
    /// coordinate.
    pub fn neighbors(&self) -> impl Iterator<Item = Coord> {
        let copy = *self;
        self.row.indexes()
            .chain(self.col.indexes())
            .chain(
                self.sector().indexes()
                    .filter(move |other| other.row != copy.row && other.col != copy.col)
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

    type All = Coords;

    fn all() -> Self::All {
        Coords(0..Board::SIZE)
    }

    type Indexes = std::iter::Once<Coord>;

    #[inline]
    fn indexes(&self) -> Self::Indexes {
        std::iter::once(*self)
    }
}

/// Iterator over all coordinates.
pub struct Coords(Range<usize>);

impl Coords {
    fn build_zone(idx: usize) -> Coord {
        Coord::from_flat_index(idx)
    }
}

zone_all_iter!(Coords, Coord);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coord_iter() {
        for r in 0..9 {
            for c in 0..9 {
                let coord = Coord::new(r, c);
                let result: Vec<_> = coord.indexes().collect();
                assert_eq!(result, vec![coord]);
            }
        }
    }
}