use std::iter::FusedIterator;
use std::ops::Range;

use crate::collections::indexed::FixedSizeIndex;
use crate::coordinates::{ZoneContaining, FixedSizeIndexable};
use crate::{Col, Coord, Row, Zone, SectorRow, SectorCol};

/// Identifies a single 3x3 sector on the sudoku board.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Sector {
    /// Row (y) index of the sector out of 3.
    base_row: u8,
    /// Column (x) index of the sector out of 3.
    base_col: u8,
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

    #[inline]
    pub(crate) fn base_row(&self) -> u8 {
        self.base_row
    }

    #[inline]
    pub(crate) fn base_col(&self) -> u8 {
        self.base_col
    }

    #[inline]
    pub(crate) fn row_range(&self) -> Range<u8> {
        self.base_row..self.base_row + Self::HEIGHT
    }

    #[inline]
    pub(crate) fn col_range(&self) -> Range<u8> {
        self.base_col..self.base_col + Self::WIDTH
    }

    /// Rows within this sector.
    pub fn rows(
        &self,
    ) -> impl Iterator<Item = SectorRow> + DoubleEndedIterator + ExactSizeIterator + FusedIterator {
        let copy = *self;
        (0..Self::HEIGHT).map(move |r| SectorRow::new(copy, r))
    }

    /// Cols within this sector.
    pub fn cols(
        &self,
    ) -> impl Iterator<Item = SectorCol> + DoubleEndedIterator + ExactSizeIterator + FusedIterator {
        let copy = *self;
        (0..Self::WIDTH).map(move |c| SectorCol::new(copy, c))
    }
}

impl FixedSizeIndexable for Sector {
    type Item = Coord;

    const NUM_ITEMS: usize = (Self::WIDTH * Self::HEIGHT) as usize;

    fn get_at_index(&self, idx: usize) -> Self::Item {
        assert!(idx < Self::NUM_ITEMS, "index {} out of range", idx);
        let idx = idx as u8;
        let row_offset = idx / Self::WIDTH;
        let col_offset = idx % Self::WIDTH;
        Coord::new(self.base_row + row_offset, self.base_col + col_offset)
    }
}

impl ZoneContaining for Sector {
    fn containing_zone(coord: impl Into<Coord>) -> Self {
        let coord = coord.into();
        // Truncate relative row by integer division then multiplication.
        Sector {
            base_row: coord.row().inner() / Self::HEIGHT * Self::HEIGHT,
            base_col: coord.col().inner() / Self::WIDTH * Self::WIDTH,
        }
    }
}

impl FixedSizeIndex for Sector {
    const NUM_INDEXES: usize = Self::NUM_SECTORS as usize;

    fn idx(&self) -> usize {
        // NOTE: We know that base_row = row * SECTORS_ACROSS. Otherwise this would be:
        // self.base_row / Self::HEIGHT * SECTORS_ACROSS + self.base_col / Self::WIDTH
        // The compiler cannot prove that base_row will be an exact multiple of HEIGHT,
        // but we can.
        (self.base_row + self.base_col / Self::WIDTH) as usize
    }

    fn from_idx(idx: usize) -> Self {
        assert!(
            idx < Self::NUM_INDEXES,
            "flat index must be in range [0, {}), got {}",
            Self::NUM_INDEXES,
            idx
        );
        let idx = idx as u8;
        // Again, this logic is based on knowing that SECTORS_ACROSS = HEIGHT. It would be 
        // wrong if those didn't match, just as in idx().
        let col = idx % Self::SECTORS_ACROSS;
        Sector { 
            base_row: idx - col,
            base_col: col * Self::WIDTH,
        }
    }
}

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
