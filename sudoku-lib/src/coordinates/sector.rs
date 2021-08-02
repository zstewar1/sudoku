use std::ops::Range;

use crate::{Coord, Row, Col, Zone};

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
    const WIDTH: u8 = 3;
    /// Height of a sector in rows.
    const HEIGHT: u8 = 3;

    /// Total number of sectors.
    const NUM_SECTORS: u8 = (Row::SIZE as u8 / Self::WIDTH) * (Col::SIZE as u8 / Self::HEIGHT);

    /// Get the sector containing the given coordinate.
    pub(crate) fn containing(coord: impl Into<Coord>) -> Self {
        let coord = coord.into();
        Sector {
            row: coord.row().inner() / Self::HEIGHT,
            col: coord.col().inner() / Self::WIDTH,
        }
    }
}

impl Zone for Sector {
    type All = Sectors;

    fn all() -> Self::All {
        Sectors(0..Self::NUM_SECTORS)
    }

    type Indexes = Indexes;

    #[inline]
    fn indexes(&self) -> Self::Indexes {
        Indexes {
            range: 0..9,
            base_row: self.row * Self::HEIGHT, 
            base_col: self.col * Self::WIDTH, 
        }
    }
}

pub struct Indexes {
    range: Range<u8>,
    base_row: u8,
    base_col: u8,
}

impl Indexes {
    #[inline]
    fn build_coord(&self, idx: u8) -> Coord {
        // Effectively converting back from row-major form, so we use width for
        // both.
        let row_offset = idx / Sector::WIDTH;
        let col_offset = idx % Sector::WIDTH;
        Coord::new(self.base_row + row_offset, self.base_col + col_offset)
    }
}

zone_indexes_iter!(Indexes);

/// Iterator over all sectors.
pub struct Sectors(Range<u8>);

impl Sectors {
    fn build_zone(idx: u8) -> Sector {
        const SECTORS_ACROSS: u8 = Row::SIZE as u8 / Sector::WIDTH;
        let row = idx / SECTORS_ACROSS;
        let col = idx % SECTORS_ACROSS;
        Sector { row, col }
    }
}

zone_all_iter!(Sectors, Sector);

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
                static OFFSETS: &[(u8, u8)] = &[(0,0), (0,1), (0,2), (1,0), (1,1), (1,2), (2,0), (2,1), (2,2)];
                let expected: Vec<_> = OFFSETS.iter().map(|&(offr, offc)| Coord::new(baser + offr, basec + offc)).collect();
                let result: Vec<_> = sector.indexes().collect();
                assert_eq!(result, expected);
            }
        }
    }
}