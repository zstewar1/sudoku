use crate::{Sector, Coord, Row, Intersect, SectorCol, Col};
use crate::coordinates::{FixedSizeIndexable, ZoneContaining};
use crate::collections::indexed::FixedSizeIndex;

/// A row within a sector.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Hash)]
pub struct SectorRow {
    /// The sector.
    sector: Sector,
    /// The row relative to the sector.
    rel_row: u8,
}

impl SectorRow {
    #[inline]
    pub(in crate::coordinates) fn new(sector: Sector, rel_row: u8) -> Self {
        SectorRow {
            sector,
            rel_row,
        }
    }

    /// Get the sector that this row is part of.
    #[inline]
    pub fn sector(&self) -> Sector {
        self.sector
    }

    /// Get the row that this row is part of.
    #[inline]
    pub fn row(&self) -> Row {
        Row::new(self.sector.base_row() + self.rel_row)
    }
}

impl FixedSizeIndexable for SectorRow {
    type Item = Coord;

    const NUM_ITEMS: usize = Sector::WIDTH as usize;

    fn get_at_index(&self, idx: usize) -> Self::Item {
        assert!(idx < Self::NUM_ITEMS, "index {} out of range", idx);
        let row = self.sector.base_row() + self.rel_row;
        let col = self.sector.base_col() + idx as u8;
        Coord::new(row, col)
    }
}

impl ZoneContaining for SectorRow {
    #[inline]
    fn containing_zone(coord: impl Into<Coord>) -> Self {
        let coord = coord.into();
        let sector = Sector::containing_zone(coord);
        let rel_row = coord.row().inner() - sector.base_row();
        SectorRow {
            sector,
            rel_row,
        }
    }
}

impl FixedSizeIndex for SectorRow {
    const NUM_INDEXES: usize = (Sector::NUM_SECTORS * Sector::HEIGHT) as usize;

    fn idx(&self) -> usize {
        self.sector.idx() * Sector::HEIGHT as usize + self.rel_row as usize
    }

    fn from_idx(idx: usize) -> Self {
        assert!(
            idx < Self::NUM_INDEXES,
            "flat index must be in range [0, {}), got {}",
            Self::NUM_INDEXES,
            idx
        );
        let sector = idx / Sector::HEIGHT as usize;
        let rel_row = idx % Sector::HEIGHT as usize;
        SectorRow {
            sector: Sector::from_idx(sector),
            rel_row: rel_row as u8,
        }
    }
}

impl Intersect<Row> for Sector {
    type Intersection = SectorRow;

    fn intersect(self, row: Row) -> Option<Self::Intersection> {
        if self.row_range().contains(&row.inner()) {
            Some(SectorRow {
                sector: self,
                rel_row: row.inner() - self.base_row(),
            })
        } else {
            None
        }
    }
}

impl Intersect<Row> for SectorRow {
    type Intersection = SectorRow;

    fn intersect(self, row: Row) -> Option<Self::Intersection> {
        if self.row() == row {
            Some(self)
        } else {
            None
        }
    }
}

impl Intersect<Col> for SectorRow {
    type Intersection = Coord;

    fn intersect(self, col: Col) -> Option<Self::Intersection> {
        if self.sector.col_range().contains(&col.inner()) {
            Some(Coord::new(self.row(), col))
        } else {
            None
        }
    }
}

impl Intersect<Sector> for SectorRow {
    type Intersection = SectorRow;

    fn intersect(self, sector: Sector) -> Option<Self::Intersection> {
        if self.sector == sector {
            Some(self)
        } else {
            None
        }
    }
}

impl Intersect<SectorCol> for SectorRow {
    type Intersection = Coord;

    fn intersect(self, other: SectorCol) -> Option<Self::Intersection> {
        if self.sector == other.sector() {
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