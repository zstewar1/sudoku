use crate::Coord;
use crate::collections::indexed::{Values, FixedSizeIndex};

/// A zone of the board is an area that must uniquely contain all numbers 1-9.
/// This is an abstraction over row, column, and sector.
pub trait Zone {
    /// Number of coordinates in this zone.
    const SIZE: usize = 9;

    /// Get an iterator over all values of this zone.
    fn all() -> Values<Self> where Self: FixedSizeIndex + Sized {
        Self::values()
    }

    /// Type used for the index iterator.
    type Coords: Iterator<Item = Coord>;

    /// Get an iterator over the coordinates of this zone.
    fn coords(&self) -> Self::Coords;

    /// Gets the zone of this type which contains the given coordinate.
    fn containing(coord: impl Into<Coord>) -> Self;

    /// True if the given coordinate is in this zone.
    fn contains(&self, coord: impl Into<Coord>) -> bool;

    /// Get the intersection between two zones.
    fn intersect<Z: Zone>(self, other: Z) -> Intersect<Self, Z>
    where
        Self: Sized,
    {
        Intersect {
            iter: self.coords(),
            z1: self,
            z2: other,
        }
    }

    /// Get the union of two zones.
    fn union<Z: Zone>(self, other: Z) -> Union<Self, Z>
    where
        Self: Sized,
    {
        Union {
            iter1: self.coords(),
            iter2: other.coords(),
            z1: self,
            z2: other,
        }
    }

    /// Get the difference between two zones.
    fn difference<Z: Zone>(self, other: Z) -> Difference<Self, Z>
    where
        Self: Sized,
    {
        Difference {
            iter: self.coords(),
            z1: self,
            z2: other,
        }
    }
}



/// Intersection between two zones. Iterator over all coordinates in the intersection.
pub struct Intersect<Z1: Zone, Z2: Zone> {
    iter: <Z1 as Zone>::Coords,
    z1: Z1,
    z2: Z2,
}

impl<Z1: Zone, Z2: Zone> Intersect<Z1, Z2> {
    /// True if the intersection contains the point.
    #[inline]
    pub fn contains(&self, coord: impl Into<Coord>) -> bool {
        let coord = coord.into();
        self.z1.contains(coord) && self.z2.contains(coord)
    }
}

impl<Z1: Zone, Z2: Zone> Iterator for Intersect<Z1, Z2> {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.iter.next() {
            if self.z2.contains(next) {
                return Some(next);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, max) = self.iter.size_hint();
        (0, max)
    }
}

impl<Z1: Zone, Z2: Zone> DoubleEndedIterator for Intersect<Z1, Z2>
where
    Z1::Coords: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.iter.next_back() {
            if self.z2.contains(next) {
                return Some(next);
            }
        }
        None
    }
}

/// Union between two zones. Iterator over all coordinates in both zones.
pub struct Union<Z1: Zone, Z2: Zone> {
    iter1: <Z1 as Zone>::Coords,
    iter2: <Z2 as Zone>::Coords,
    z1: Z1,
    z2: Z2,
}

impl<Z1: Zone, Z2: Zone> Union<Z1, Z2> {
    /// True if the union contains the point.
    #[inline]
    pub fn contains(&self, coord: impl Into<Coord>) -> bool {
        let coord = coord.into();
        self.z1.contains(coord) || self.z2.contains(coord)
    }
}

impl<Z1: Zone, Z2: Zone> Iterator for Union<Z1, Z2> {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.iter1.next() {
            return Some(next);
        }
        while let Some(next) = self.iter2.next() {
            if !self.z1.contains(next) {
                return Some(next);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (min, max1) = self.iter1.size_hint();
        let (_, max2) = self.iter2.size_hint();
        let max = match (max1, max2) {
            (Some(max1), Some(max2)) => Some(max1 + max2),
            _ => None,
        };
        (min, max)
    }
}

impl<Z1: Zone, Z2: Zone> DoubleEndedIterator for Union<Z1, Z2>
where
    Z1::Coords: DoubleEndedIterator,
    Z2::Coords: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.iter2.next_back() {
            if !self.z1.contains(next) {
                return Some(next);
            }
        }
        self.iter1.next_back()
    }
}

/// Difference between two zones. Iterator over all coordinates in Z1 that are
/// not in Z2.
pub struct Difference<Z1: Zone, Z2: Zone> {
    iter: <Z1 as Zone>::Coords,
    z1: Z1,
    z2: Z2,
}

impl<Z1: Zone, Z2: Zone> Difference<Z1, Z2> {
    /// True if the intersection contains the point.
    #[inline]
    pub fn contains(&self, coord: impl Into<Coord>) -> bool {
        let coord = coord.into();
        self.z1.contains(coord) && !self.z2.contains(coord)
    }
}

impl<Z1: Zone, Z2: Zone> Iterator for Difference<Z1, Z2> {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.iter.next() {
            if !self.z2.contains(next) {
                return Some(next);
            }
        }
        None
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, max) = self.iter.size_hint();
        (0, max)
    }
}

impl<Z1: Zone, Z2: Zone> DoubleEndedIterator for Difference<Z1, Z2>
where
    Z1::Coords: DoubleEndedIterator,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.iter.next_back() {
            if !self.z2.contains(next) {
                return Some(next);
            }
        }
        None
    }
}
