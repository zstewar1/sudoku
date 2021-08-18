use std::iter::FusedIterator;

use crate::collections::indexed::FixedSizeIndex;
use crate::coordinates::{FixedSizeIndexable, ZoneContaining};
use crate::{Col, Coord, Intersect, Row, Sector};

/// A column within a sector.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct SectorCol {
    /// The row that the sector starts at.
    base_row: Row,
    /// The column.
    col: Col,
}

impl SectorCol {
    /// Create a sector-col with the given base.
    #[inline]
    pub(in crate::coordinates) fn new(base_row: Row, col: Col) -> Self {
        debug_assert!(base_row.sector_base() == base_row);
        Self { base_row, col }
    }

    /// Get the sector that this col is part of.
    #[inline]
    pub fn sector(&self) -> Sector {
        Sector::containing_zone(Coord::new(self.base_row, self.col))
    }

    /// Get the row that this row is part of.
    #[inline]
    pub fn col(&self) -> Col {
        self.col
    }

    /// Get the base row for the sector that this row is part of.
    #[inline]
    pub fn base_row(&self) -> Row {
        self.base_row
    }

    /// Iterator over all SectorCols in the rest of the sector and column.
    pub fn neighbors(
        self,
    ) -> impl Iterator<Item = SectorCol> + DoubleEndedIterator + FusedIterator {
        self.col()
            .sector_cols()
            .chain(self.sector().cols())
            .filter(move |sr| *sr != self)
    }
}

impl FixedSizeIndexable for SectorCol {
    type Item = Coord;

    const NUM_ITEMS: usize = Sector::HEIGHT as usize;

    fn get_at_index(&self, idx: usize) -> Self::Item {
        assert!(idx < Self::NUM_ITEMS, "index {} out of range", idx);
        let row = self.base_row.inner() + idx as u8;
        Coord::new(Row::new(row), self.col)
    }
}

fixed_size_indexable_into_iter!(SectorCol);

impl ZoneContaining for SectorCol {
    #[inline]
    fn containing_zone(coord: Coord) -> Self {
        SectorCol {
            base_row: coord.row().sector_base(),
            col: coord.col(),
        }
    }
}

impl FixedSizeIndex for SectorCol {
    const NUM_INDEXES: usize = (Sector::NUM_SECTORS * Sector::WIDTH) as usize;

    fn idx(&self) -> usize {
        // We want to do base_row / HEIGHT * NUM_INDEXES, so pre-compute
        // NUM_INDEXES / HEIGHT.
        const ROW_FACTOR: u8 = Col::NUM_INDEXES as u8 / Sector::HEIGHT;
        let row = self.base_row.inner() * ROW_FACTOR;
        row as usize + self.col.idx()
    }

    fn from_idx(idx: usize) -> Self {
        assert!(
            idx < Self::NUM_INDEXES,
            "flat index must be in range [0, {}), got {}",
            Self::NUM_INDEXES,
            idx
        );
        const ROW_FACTOR: u8 = Col::NUM_INDEXES as u8 / Sector::HEIGHT;
        let col = (idx % Col::NUM_INDEXES) as u8;
        let row = (idx as u8 - col) / ROW_FACTOR;
        SectorCol {
            base_row: Row::new(row),
            col: Col::new(col),
        }
    }
}

impl Intersect<Col> for Sector {
    type Intersection = SectorCol;

    fn intersect(self, col: Col) -> Option<Self::Intersection> {
        if self.base_col() == col.sector_base() {
            Some(SectorCol {
                base_row: self.base_row(),
                col,
            })
        } else {
            None
        }
    }
}

impl Intersect<Col> for SectorCol {
    type Intersection = SectorCol;

    fn intersect(self, col: Col) -> Option<Self::Intersection> {
        if self.col == col {
            Some(self)
        } else {
            None
        }
    }
}

impl Intersect<Row> for SectorCol {
    type Intersection = Coord;

    fn intersect(self, row: Row) -> Option<Self::Intersection> {
        if self.base_row == row.sector_base() {
            Some(Coord::new(row, self.col))
        } else {
            None
        }
    }
}

impl Intersect<Sector> for SectorCol {
    type Intersection = SectorCol;

    fn intersect(self, sector: Sector) -> Option<Self::Intersection> {
        if sector.base_row() == self.base_row && sector.base_col() == self.col.sector_base() {
            Some(self)
        } else {
            None
        }
    }
}

reciprocal_intersect!(<Sector> for Col);
reciprocal_intersect!(<SectorCol> for Col);
reciprocal_intersect!(<SectorCol> for Row);
reciprocal_intersect!(<SectorCol> for Sector);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Zone;

    #[test]
    fn colsec_iter() {
        for br in (0..9).step_by(3) {
            for c in 0..9 {
                let seccol = SectorCol {
                    base_row: Row::new(br),
                    col: Col::new(c),
                };
                let mut expected = Vec::with_capacity(3);
                for r in (br..).take(3) {
                    expected.push(Coord::new(Row::new(r), Col::new(c)));
                }
                let result: Vec<_> = seccol.coords().collect();
                assert_eq!(result, expected);
            }
        }
    }

    #[test]
    fn colsecs_iter() {
        let mut expected = Vec::with_capacity(27);
        for br in (0..9).step_by(3) {
            for c in 0..9 {
                expected.push(SectorCol {
                    base_row: Row::new(br),
                    col: Col::new(c),
                });
            }
        }
        let result: Vec<_> = SectorCol::values().collect();
        assert_eq!(result, expected);
        for (idx, val) in result.iter().enumerate() {
            assert_eq!(val.idx(), idx);
        }
    }

    #[test]
    fn colsec_neighbors() {
        for br in (0..9).step_by(3) {
            for c in 0..9 {
                let seccol = SectorCol {
                    base_row: Row::new(br),
                    col: Col::new(c),
                };
                let mut expected = Vec::with_capacity(4);
                for brr in (0..9).step_by(3) {
                    if brr != br {
                        expected.push(SectorCol {
                            base_row: Row::new(brr),
                            col: Col::new(c),
                        });
                    }
                }
                for cc in (c - c % 3..).take(3) {
                    if cc != c {
                        expected.push(SectorCol {
                            base_row: Row::new(br),
                            col: Col::new(cc),
                        });
                    }
                }
                let result: Vec<_> = seccol.neighbors().collect();
                assert_eq!(result.len(), 4);
                assert_eq!(result, expected);
            }
        }
    }
}
