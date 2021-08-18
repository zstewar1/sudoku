use std::iter::FusedIterator;

use crate::collections::indexed::FixedSizeIndex;
use crate::coordinates::{FixedSizeIndexable, ZoneContaining};
use crate::{Col, Coord, Intersect, Row, Sector};

/// A column within a sector.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct SectorCol {
    /// The sector.
    sector: Sector,
    /// The column relative to the sector.
    rel_col: u8,
}

impl SectorCol {
    #[inline]
    pub(in crate::coordinates) fn new(sector: Sector, rel_col: u8) -> Self {
        SectorCol { sector, rel_col }
    }

    /// Get the sector that this col is part of.
    #[inline]
    pub fn sector(&self) -> Sector {
        self.sector
    }

    /// Get the row that this row is part of.
    #[inline]
    pub fn col(&self) -> Col {
        Col::new(self.sector.base_col() + self.rel_col)
    }

    /// Iterator over all SectorCols in the rest of the sector and column.
    pub fn neighbors(
        self,
    ) -> impl Iterator<Item = SectorCol> + DoubleEndedIterator + FusedIterator {
        self.col()
            .sector_cols()
            .chain(self.sector.cols())
            .filter(move |sr| *sr != self)
    }
}

impl FixedSizeIndexable for SectorCol {
    type Item = Coord;

    const NUM_ITEMS: usize = Sector::HEIGHT as usize;

    fn get_at_index(&self, idx: usize) -> Self::Item {
        assert!(idx < Self::NUM_ITEMS, "index {} out of range", idx);
        let row = self.sector.base_row() + idx as u8;
        let col = self.sector.base_col() + self.rel_col;
        Coord::new(Row::new(row), Col::new(col))
    }
}

fixed_size_indexable_into_iter!(SectorCol);

impl ZoneContaining for SectorCol {
    #[inline]
    fn containing_zone(coord: Coord) -> Self {
        let coord = coord.into();
        let sector = Sector::containing_zone(coord);
        let rel_col = coord.col().inner() - sector.base_col();
        SectorCol { sector, rel_col }
    }
}

impl FixedSizeIndex for SectorCol {
    const NUM_INDEXES: usize = (Sector::NUM_SECTORS * Sector::WIDTH) as usize;

    fn idx(&self) -> usize {
        self.sector.idx() * Sector::WIDTH as usize + self.rel_col as usize
    }

    fn from_idx(idx: usize) -> Self {
        assert!(
            idx < Self::NUM_INDEXES,
            "flat index must be in range [0, {}), got {}",
            Self::NUM_INDEXES,
            idx
        );
        let sector = idx / Sector::WIDTH as usize;
        let rel_col = idx % Sector::WIDTH as usize;
        SectorCol {
            sector: Sector::from_idx(sector),
            rel_col: rel_col as u8,
        }
    }
}

impl Intersect<Col> for Sector {
    type Intersection = SectorCol;

    fn intersect(self, col: Col) -> Option<Self::Intersection> {
        if self.col_range().contains(&col.inner()) {
            Some(SectorCol {
                sector: self,
                rel_col: col.inner() - self.base_col(),
            })
        } else {
            None
        }
    }
}

impl Intersect<Col> for SectorCol {
    type Intersection = SectorCol;

    fn intersect(self, col: Col) -> Option<Self::Intersection> {
        if self.col() == col {
            Some(self)
        } else {
            None
        }
    }
}

impl Intersect<Row> for SectorCol {
    type Intersection = Coord;

    fn intersect(self, row: Row) -> Option<Self::Intersection> {
        if self.sector.row_range().contains(&row.inner()) {
            Some(Coord::new(row, self.col()))
        } else {
            None
        }
    }
}

impl Intersect<Sector> for SectorCol {
    type Intersection = SectorCol;

    fn intersect(self, sector: Sector) -> Option<Self::Intersection> {
        if self.sector == sector {
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
            for bc in (0..9).step_by(3) {
                for rc in 0..3 {
                    let seccol = SectorCol {
                        sector: Sector::new_unchecked(br, bc),
                        rel_col: rc,
                    };
                    let mut expected = Vec::with_capacity(3);
                    for r in (br..).take(3) {
                        expected.push(Coord::new(Row::new(r), Col::new(bc + rc)));
                    }
                    let result: Vec<_> = seccol.coords().collect();
                    assert_eq!(result, expected);
                }
            }
        }
    }

    #[test]
    fn colsecs_iter() {
        let mut expected = Vec::with_capacity(27);
        for br in (0..9).step_by(3) {
            for bc in (0..9).step_by(3) {
                for rc in 0..3 {
                    expected.push(SectorCol {
                        sector: Sector::new_unchecked(br, bc),
                        rel_col: rc,
                    });
                }
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
            for bc in (0..9).step_by(3) {
                for rc in 0..3 {
                    let seccol = SectorCol {
                        sector: Sector::new_unchecked(br, bc),
                        rel_col: rc,
                    };
                    let mut expected = Vec::with_capacity(4);
                    for brr in (0..9).step_by(3) {
                        if brr != br {
                            expected.push(SectorCol {
                                sector: Sector::new_unchecked(brr, bc),
                                rel_col: rc,
                            });
                        }
                    }
                    for rcc in 0..3 {
                        if rcc != rc {
                            expected.push(SectorCol {
                                sector: Sector::new_unchecked(br, bc),
                                rel_col: rcc,
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
}
