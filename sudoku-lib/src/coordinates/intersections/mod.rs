use crate::{Col, Coord, Row, Sector, Zone, SectorRow, SectorCol};

pub(crate) mod rowsec;
pub(crate) mod colsec;

/// Trait for the intersection of a zone with another type of zone.
pub trait Intersect<Z: Zone> {
    type Intersection: Zone;

    /// Get the intersection of this zone with the given other zone.
    fn intersect(self, other: Z) -> Option<Self::Intersection>;
}

impl<Z: Zone + PartialEq> Intersect<Z> for Z {
    type Intersection = Self;

    fn intersect(self, other: Z) -> Option<Self::Intersection> {
        if self == other {
            Some(other)
        } else {
            None
        }
    }
}

macro_rules! coord_zone_intersect {
    ($z:ty) => {
        impl Intersect<$z> for Coord {
            type Intersection = Self;

            fn intersect(self, other: $z) -> Option<Self::Intersection> {
                if other.contains(self) {
                    Some(self)
                } else {
                    None
                }
            }
        }

        reciprocal_intersect!(<Coord> for $z);
    };
}

coord_zone_intersect!(Row);
coord_zone_intersect!(Col);
coord_zone_intersect!(Sector);
coord_zone_intersect!(SectorRow);
coord_zone_intersect!(SectorCol);

impl Intersect<Col> for Row {
    type Intersection = Coord;

    fn intersect(self, other: Col) -> Option<Self::Intersection> {
        Some(Coord::new(self, other))
    }
}

reciprocal_intersect!(<Row> for Col);
