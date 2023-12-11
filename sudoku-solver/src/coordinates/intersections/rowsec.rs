use std::array;
use std::iter::FusedIterator;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::collections::indexed::FixedSizeIndex;
use crate::coordinates::{FixedSizeIndexable, ZoneContaining};
use crate::{Col, Coord, Intersect, Row, Sector, SectorCol};

/// A row within a sector.
/// Sector rows sort in the same order as their equivalent indexes, by row then
/// by column (so across the rows).
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SectorRow {
    /// The row relative to the sector.
    row: Row,
    #[cfg_attr(
        feature = "serde",
        serde(deserialize_with = "crate::coordinates::serde_utils::deserialize_base_col")
    )]
    base_col: Col,
}

impl SectorRow {
    #[inline]
    pub(in crate::coordinates) fn new(row: Row, base_col: Col) -> Self {
        debug_assert!(base_col.sector_base() == base_col);
        SectorRow { row, base_col }
    }

    /// Get the sector that this row is part of.
    #[inline]
    pub fn sector(&self) -> Sector {
        Sector::containing_zone(Coord::new(self.row, self.base_col))
    }

    /// Get the row that this row is part of.
    #[inline]
    pub fn row(&self) -> Row {
        self.row
    }

    /// Get the base col for the sector that this col is part of.
    #[inline]
    pub fn base_col(&self) -> Col {
        self.base_col
    }

    /// Gets an iterator over the two SectorRows that share the same row as this one.
    #[inline]
    pub fn row_neighbors(self) -> array::IntoIter<Self, 2> {
        super::array_filter_single_neq(self, self.row.sector_rows()).into_iter()
    }

    /// Gets an iterator over the two SectorRows that share the same sector as this one.
    #[inline]
    pub fn sector_neighbors(self) -> array::IntoIter<Self, 2> {
        super::array_filter_single_neq(self, self.sector().rows()).into_iter()
    }

    /// Iterator over all SectorRows in the rest of the sector and row.
    pub fn neighbors(
        self,
    ) -> impl Iterator<Item = SectorRow> + DoubleEndedIterator + FusedIterator {
        self.row()
            .sector_rows()
            .chain(self.sector().rows())
            .filter(move |sr| *sr != self)
    }
}

impl FixedSizeIndexable for SectorRow {
    type Item = Coord;

    const NUM_ITEMS: usize = Sector::WIDTH as usize;

    fn get_at_index(&self, idx: usize) -> Self::Item {
        assert!(idx < Self::NUM_ITEMS, "index {} out of range", idx);
        let col = self.base_col.inner() + idx as u8;
        Coord::new(self.row, Col::new(col))
    }
}

fixed_size_indexable_into_iter!(SectorRow);

impl ZoneContaining for SectorRow {
    #[inline]
    fn containing_zone(coord: Coord) -> Self {
        SectorRow {
            row: coord.row(),
            base_col: coord.col().sector_base(),
        }
    }
}

impl FixedSizeIndex for SectorRow {
    const NUM_INDEXES: usize = (Sector::NUM_SECTORS * Sector::HEIGHT) as usize;

    fn idx(&self) -> usize {
        let row = self.row.inner() * Sector::SECTORS_ACROSS;
        let col = self.base_col.inner() / Sector::WIDTH;
        (row + col) as usize
    }

    fn from_idx(idx: usize) -> Self {
        assert!(
            idx < Self::NUM_INDEXES,
            "flat index must be in range [0, {}), got {}",
            Self::NUM_INDEXES,
            idx
        );
        let idx = idx as u8;
        let row = idx / Sector::SECTORS_ACROSS;
        let col = (idx % Sector::SECTORS_ACROSS) * Sector::WIDTH;
        SectorRow {
            row: Row::new(row),
            base_col: Col::new(col),
        }
    }
}

impl Intersect<Row> for Sector {
    type Intersection = SectorRow;

    fn intersect(self, row: Row) -> Option<Self::Intersection> {
        if self.base_row() == row.sector_base() {
            Some(SectorRow {
                row,
                base_col: self.base_col(),
            })
        } else {
            None
        }
    }
}

impl Intersect<Row> for SectorRow {
    type Intersection = SectorRow;

    fn intersect(self, row: Row) -> Option<Self::Intersection> {
        if self.row == row {
            Some(self)
        } else {
            None
        }
    }
}

impl Intersect<Col> for SectorRow {
    type Intersection = Coord;

    fn intersect(self, col: Col) -> Option<Self::Intersection> {
        if self.base_col == col.sector_base() {
            Some(Coord::new(self.row, col))
        } else {
            None
        }
    }
}

impl Intersect<Sector> for SectorRow {
    type Intersection = SectorRow;

    fn intersect(self, sector: Sector) -> Option<Self::Intersection> {
        if sector.base_row() == self.row.sector_base() && sector.base_col() == self.base_col {
            Some(self)
        } else {
            None
        }
    }
}

impl Intersect<SectorCol> for SectorRow {
    type Intersection = Coord;

    fn intersect(self, other: SectorCol) -> Option<Self::Intersection> {
        if self.row.sector_base() == other.base_row() && self.base_col == other.col().sector_base()
        {
            Some(Coord::new(self.row(), other.col()))
        } else {
            None
        }
    }
}

reciprocal_intersect!(<Sector> for Row);
reciprocal_intersect!(<SectorRow> for Row);
reciprocal_intersect!(<SectorRow> for Col);
reciprocal_intersect!(<SectorRow> for Sector);
reciprocal_intersect!(<SectorRow> for SectorCol);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Zone;

    #[test]
    fn rowsec_iter() {
        for r in 0..9 {
            for bc in (0..9).step_by(3) {
                let secrow = SectorRow {
                    row: Row::new(r),
                    base_col: Col::new(bc),
                };
                let mut expected = Vec::with_capacity(3);
                for c in (bc..).take(3) {
                    expected.push(Coord::new(Row::new(r), Col::new(c)));
                }
                let result: Vec<_> = secrow.coords().collect();
                assert_eq!(result, expected);
            }
        }
    }

    #[test]
    fn rowsecs_iter() {
        let mut expected = Vec::with_capacity(27);
        for r in 0..9 {
            for bc in (0..9).step_by(3) {
                expected.push(SectorRow {
                    row: Row::new(r),
                    base_col: Col::new(bc),
                });
            }
        }
        let result: Vec<_> = SectorRow::values().collect();
        assert_eq!(result, expected);
        assert_sorted!(result);
        for (idx, val) in result.iter().enumerate() {
            assert_eq!(val.idx(), idx);
        }
    }

    #[test]
    fn rowsec_neighbors() {
        for r in 0..9 {
            for bc in (0..9).step_by(3) {
                let secrow = SectorRow {
                    row: Row::new(r),
                    base_col: Col::new(bc),
                };
                let mut expected = Vec::with_capacity(4);
                for bcc in (0..9).step_by(3) {
                    if bcc != bc {
                        expected.push(SectorRow {
                            row: Row::new(r),
                            base_col: Col::new(bcc),
                        });
                    }
                }
                for rr in (r - r % 3..).take(3) {
                    if rr != r {
                        expected.push(SectorRow {
                            row: Row::new(rr),
                            base_col: Col::new(bc),
                        });
                    }
                }
                let result: Vec<_> = secrow.neighbors().collect();
                assert_eq!(result.len(), 4);
                assert_eq!(result, expected);
            }
        }
    }
}
