use crate::Coord;

/// A zone of the board is an area that must uniquely contain all numbers 1-9.
/// This is an abstraction over row, column, and sector.
pub trait Zone {

    /// Number of coordinates in this zone.
    const SIZE: usize = 9;

    /// Type used for the all iterator.
    type All: Iterator<Item = Self>;

    /// Get an iterator over all values of this zone.
    fn all() -> Self::All;

    /// Type used for the index iterator.
    type Indexes: Iterator<Item = Coord>;

    /// Get an iterator over the coordinates of this zone.
    fn indexes(&self) -> Self::Indexes;
}
