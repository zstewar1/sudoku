use std::iter::FusedIterator;
use std::ops::Range;

use crate::{Col, Coord, Row, Zone};
use crate::collections::indexed::FixedSizeIndex;

/// Identifies a single 3x3 sector on the sudoku board.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Sector {
    /// Row (y) index of the sector out of 3.
    pub(crate) row: u8,
    /// Column (x) index of the sector out of 3.
    pub(crate) col: u8,
}

impl Sector {
    /// Width of a sector in columns.
    pub(crate) const WIDTH: u8 = 3;
    /// Height of a sector in rows.
    pub(crate) const HEIGHT: u8 = 3;

    /// Number of sectors across a row. (Number of sector columns).
    pub(crate) const SECTORS_ACROSS: u8 = Row::SIZE as u8 / Self::WIDTH;

    /// Number of sectors down a column. (Number of sector rows).
    pub(crate) const SECTORS_DOWN: u8 = Col::SIZE as u8 / Self::HEIGHT;

    /// Total number of sectors.
    pub(crate) const NUM_SECTORS: u8 = Self::SECTORS_ACROSS * Self::SECTORS_DOWN;

    /// Converts a row, column, or coordinate to one relative to this sector.
    #[inline]
    pub(crate) fn to_relative<T: Relative>(&self, coord: T) -> Option<<T as Relative>::Rel> {
        coord.to_rel(self)
    }

    /// Converts a row, column, or coordinate relative to this sector back to an absolute one.
    #[inline]
    pub(crate) fn from_relative<T: Relative>(&self, coord: <T as Relative>::Rel) -> T {
        T::from_rel(coord, self)
    }

    #[inline]
    fn base_row(&self) -> u8 {
        self.row * Self::HEIGHT
    }

    #[inline]
    fn base_col(&self) -> u8 {
        self.col * Self::WIDTH
    }

    /// Rows within this sector.
    pub fn rows(&self) -> impl Iterator<Item = Row> + DoubleEndedIterator + ExactSizeIterator + FusedIterator {
        let start = self.base_row();
        let end = start + Self::HEIGHT;
        (start..end).map(|r| Row::new(r))
    }

    /// Cols within this sector.
    pub fn cols(&self) -> impl Iterator<Item = Col> + DoubleEndedIterator + ExactSizeIterator + FusedIterator {
        let start = self.base_col();
        let end = start + Self::WIDTH;
        (start..end).map(|c| Col::new(c))
    }
}

/// Trait for getting row/col/coord relative to a sector.
pub trait Relative {
    type Rel;

    fn to_rel(self, sec: &Sector) -> Option<Self::Rel>;

    fn from_rel(rel: Self::Rel, sec: &Sector) -> Self;
}

impl Relative for Row {
    type Rel = u8;

    #[inline]
    fn to_rel(self, sec: &Sector) -> Option<Self::Rel> {
        let base = sec.base_row();
        if (base..base + Sector::HEIGHT).contains(&self.inner()) {
            Some(self.inner() - base)
        } else {
            None
        }
    }

    #[inline]
    fn from_rel(rel: Self::Rel, sec: &Sector) -> Self {
        (rel + sec.base_row()).into()
    }
}

impl Relative for Col {
    type Rel = u8;

    #[inline]
    fn to_rel(self, sec: &Sector) -> Option<Self::Rel> {
        let base = sec.base_col();
        if (base..base + Sector::WIDTH).contains(&self.inner()) {
            Some(self.inner() - base)
        } else {
            None
        }
    }

    #[inline]
    fn from_rel(rel: Self::Rel, sec: &Sector) -> Self {
        (rel + sec.base_col()).into()
    }
}

impl Relative for Coord {
    type Rel = (u8, u8);

    #[inline]
    fn to_rel(self, sec: &Sector) -> Option<Self::Rel> {
        match (self.row().to_rel(sec), self.col().to_rel(sec)) {
            (Some(row), Some(col)) => Some((row, col)),
            _ => None,
        }
    }

    #[inline]
    fn from_rel((relr, relc): Self::Rel, sec: &Sector) -> Self {
        (Row::from_rel(relr, sec), Col::from_rel(relc, sec)).into()
    }
}

impl Zone for Sector {
    type Coords = Coords;

    #[inline]
    fn coords(&self) -> Self::Coords {
        Coords {
            range: 0..9,
            base_row: self.base_row(),
            base_col: self.base_col(),
        }
    }

    fn containing(coord: impl Into<Coord>) -> Self {
        let coord = coord.into();
        Sector {
            row: coord.row().inner() / Self::HEIGHT,
            col: coord.col().inner() / Self::WIDTH,
        }
    }

    #[inline]
    fn contains(&self, coord: impl Into<Coord>) -> bool {
        *self == Self::containing(coord)
    }
}

impl FixedSizeIndex for Sector {
    const NUM_INDEXES: usize = Self::NUM_SECTORS as usize;

    fn idx(&self) -> usize {
        (self.row * Self::SECTORS_ACROSS + self.col) as usize
    }

    fn from_idx(idx: usize) -> Self {
        assert!(
            idx < Self::NUM_INDEXES,
            "flat index must be in range [0, {}), got {}",
            Self::NUM_INDEXES,
            idx
        );
        let idx = idx as u8;
        let row = idx / Self::SECTORS_ACROSS;
        let col = idx % Self::SECTORS_ACROSS;
        Sector { row, col }
    }
}

pub struct Coords {
    range: Range<u8>,
    base_row: u8,
    base_col: u8,
}

impl Coords {
    #[inline]
    fn build_coord(&self, idx: u8) -> Coord {
        // Effectively converting back from row-major form, so we use width for
        // both.
        let row_offset = idx / Sector::WIDTH;
        let col_offset = idx % Sector::WIDTH;
        Coord::new(self.base_row + row_offset, self.base_col + col_offset)
    }
}

zone_coords_iter!(Coords);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sector_iter() {
        for r in 0..9 {
            for c in 0..9 {
                let sector = Sector::containing((r, c));
                let baser = match r {
                    0 | 1 | 2 => 0,
                    3 | 4 | 5 => 3,
                    6 | 7 | 8 => 6,
                    _ => unreachable!(),
                };
                let basec = match c {
                    0 | 1 | 2 => 0,
                    3 | 4 | 5 => 3,
                    6 | 7 | 8 => 6,
                    _ => unreachable!(),
                };
                static OFFSETS: &[(u8, u8)] = &[
                    (0, 0),
                    (0, 1),
                    (0, 2),
                    (1, 0),
                    (1, 1),
                    (1, 2),
                    (2, 0),
                    (2, 1),
                    (2, 2),
                ];
                let expected: Vec<_> = OFFSETS
                    .iter()
                    .map(|&(offr, offc)| Coord::new(baser + offr, basec + offc))
                    .collect();
                let result: Vec<_> = sector.coords().collect();
                assert_eq!(result, expected);
            }
        }
    }
}
