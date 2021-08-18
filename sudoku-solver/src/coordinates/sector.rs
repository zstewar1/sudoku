use std::iter::FusedIterator;

use crate::collections::indexed::FixedSizeIndex;
use crate::coordinates::{FixedSizeIndexable, ZoneContaining};
use crate::{Col, Coord, Row, SectorCol, SectorRow, Zone};

/// Identifies a single 3x3 sector on the sudoku board.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct Sector {
    /// Row (y) where the sector starts.
    base_row: Row,
    /// Column (x) where the sector starts.
    base_col: Col,
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
    pub(crate) fn base_row(&self) -> Row {
        self.base_row
    }

    #[inline]
    pub(crate) fn base_col(&self) -> Col {
        self.base_col
    }

    /// Rows within this sector.
    pub fn rows(
        &self,
    ) -> impl Iterator<Item = SectorRow> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        let base_col = self.base_col;
        (self.base_row.inner()..self.base_row.inner() + Self::HEIGHT)
            .map(move |r| SectorRow::new(Row::new(r), base_col))
    }

    /// Cols within this sector.
    pub fn cols(
        &self,
    ) -> impl Iterator<Item = SectorCol> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        let base_row = self.base_row;
        (self.base_col.inner()..self.base_col.inner() + Self::WIDTH)
            .map(move |c| SectorCol::new(base_row, Col::new(c)))
    }
}

impl FixedSizeIndexable for Sector {
    type Item = Coord;

    const NUM_ITEMS: usize = (Self::WIDTH * Self::HEIGHT) as usize;

    fn get_at_index(&self, idx: usize) -> Self::Item {
        assert!(idx < Self::NUM_ITEMS, "index {} out of range", idx);
        let idx = idx as u8;
        let row_offset = (idx / Self::WIDTH) as u8;
        let col_offset = (idx % Self::WIDTH) as u8;
        let row = Row::new(self.base_row.inner() + row_offset);
        let col = Col::new(self.base_col.inner() + col_offset);
        Coord::new(row, col)
    }
}

fixed_size_indexable_into_iter!(Sector);

impl ZoneContaining for Sector {
    fn containing_zone(coord: Coord) -> Self {
        // Truncate relative row by integer division then multiplication.
        Sector {
            base_row: coord.row().sector_base(),
            base_col: coord.col().sector_base(),
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
        (self.base_row.inner() + self.base_col.inner() / Self::WIDTH) as usize
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
            base_row: Row::new(idx - col),
            base_col: Col::new(col * Self::WIDTH),
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
                let sector = Sector::containing(Coord::new(Row::new(r), Col::new(c)));
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
                    .map(|&(offr, offc)| Coord::new(Row::new(baser + offr), Col::new(basec + offc)))
                    .collect();
                let result: Vec<_> = sector.coords().collect();
                assert_eq!(result, expected);
            }
        }
    }

    #[test]
    fn sectors_iter() {
        let mut expected = Vec::with_capacity(9);
        for r in (0..9).step_by(3) {
            for c in (0..9).step_by(3) {
                expected.push(Sector {
                    base_row: Row::new(r),
                    base_col: Col::new(c),
                })
            }
        }
        let result: Vec<_> = Sector::values().collect();
        assert_eq!(result, expected);
        for (idx, val) in result.iter().enumerate() {
            assert_eq!(val.idx(), idx);
        }
    }
}
