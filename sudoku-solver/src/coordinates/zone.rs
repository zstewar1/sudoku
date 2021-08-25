use std::hash::Hash;
use std::ops::Range;

use crate::collections::indexed::FixedSizeIndex;
use crate::{Coord, Values};

/// A zone of the board is an area that must uniquely contain all numbers 1-9.
/// This is an abstraction over row, column, and sector.
pub trait Zone:
    FixedSizeIndex
    + FixedSizeIndexable<Item = Coord>
    + ZoneContaining
    + PartialEq
    + Eq
    + Hash
    + Copy
    + Clone
    + IntoIterator<Item = Coord>
{
    const SIZE: usize = Self::NUM_ITEMS;

    /// Get an iterator over all values of this zone.
    #[inline]
    fn all() -> Values<Self>
    where
        Self: Sized,
    {
        Self::values()
    }

    /// Get an iterator over the coordinates of this zone.
    fn coords(&self) -> Coords<Self>
    where
        Self: Sized;

    /// Whether this zone contains the given coordinate.
    fn contains(&self, coord: Coord) -> bool;

    /// Gets the zone of this type which contains the given coordinate.
    #[inline]
    fn containing(coord: Coord) -> Self
    where
        Self: Sized,
    {
        ZoneContaining::containing_zone(coord)
    }
}

impl<Z> Zone for Z
where
    Z: FixedSizeIndex
        + FixedSizeIndexable<Item = Coord>
        + ZoneContaining
        + PartialEq
        + Eq
        + Hash
        + Copy
        + Clone
        + IntoIterator<Item = Coord>,
{
    #[inline]
    fn coords(&self) -> Coords<Self> {
        (*self).into()
    }

    #[inline]
    fn contains(&self, coord: Coord) -> bool {
        *self == Self::containing(coord)
    }
}

/// Type has a size known at compile time and can be indexed to produce a value
/// of a specific type.
pub trait FixedSizeIndexable {
    type Item;

    /// Number of items in this indexable.
    const NUM_ITEMS: usize;

    /// Get the child with the given index.
    fn get_at_index(&self, idx: usize) -> Self::Item;
}

/// Zones which can determine which zone of type Self contains a given
/// coordinate.
pub trait ZoneContaining {
    /// Gets the zone of this type which contains the given coordinate.
    fn containing_zone(coord: Coord) -> Self;
}

/// Coords of a Zone.
pub struct Coords<F> {
    range: Range<usize>,
    indexable: F,
}

impl<F: FixedSizeIndexable> From<F> for Coords<F> {
    fn from(indexable: F) -> Self {
        Coords {
            range: 0..F::NUM_ITEMS,
            indexable,
        }
    }
}

impl<F: FixedSizeIndexable> Iterator for Coords<F> {
    type Item = F::Item;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.range
            .next()
            .map(|val| self.indexable.get_at_index(val))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.range
            .nth(n)
            .map(|val| self.indexable.get_at_index(val))
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<F: FixedSizeIndexable> ExactSizeIterator for Coords<F> {}

impl<F: FixedSizeIndexable> DoubleEndedIterator for Coords<F> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.range
            .next_back()
            .map(|val| self.indexable.get_at_index(val))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.range
            .nth_back(n)
            .map(|val| self.indexable.get_at_index(val))
    }
}

impl<F: FixedSizeIndexable> std::iter::FusedIterator for Coords<F> {}
