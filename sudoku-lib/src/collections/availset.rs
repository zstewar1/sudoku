use std::iter::FusedIterator;
use std::ops::{Index, IndexMut};

/// Set of available numbers.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct AvailSet(u16);

impl AvailSet {
    /// Create a new AvailSet with all values available.
    #[inline]
    pub(crate) const fn all() -> Self {
        AvailSet(0x1ff)
    }

    /// Create an AvailSet with no values available.
    #[inline]
    pub(crate) const fn none() -> Self {
        AvailSet(0)
    }

    /// Create an AvailSet containing only the given value.
    #[inline]
    pub(crate) fn only(val: u8) -> Self {
        AvailSet(AvailSet::to_mask(val))
    }

    /// Returns true if there are no more values available.
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Returns true if this set contains a single element.
    #[inline]
    pub(crate) fn is_single(&self) -> bool {
        self.len() == 1
    }

    /// If there is only a single entry, returns that entry.
    pub(crate) fn get_single(&self) -> Option<u8> {
        if self.is_single() {
            Some((self.0.trailing_zeros() + 1) as u8)
        } else {
            None
        }
    }

    /// Add the given value to the set. Return true if the value was not in the
    /// set previously.
    pub(crate) fn add(&mut self, val: u8) -> bool {
        let added = !self.contains(val);
        self.0 |= Self::to_mask(val);
        added
    }

    /// Remove the given value from the set. Return true if the value was in the
    /// set previously.
    pub(crate) fn remove(&mut self, val: u8) -> bool {
        let had = self.contains(val);
        self.0 &= !Self::to_mask(val);
        had
    }

    /// Returns true if the set contains the given value.
    pub(crate) fn contains(&self, val: u8) -> bool {
        self.0 & Self::to_mask(val) != 0
    }

    /// Counts the number of values in this set.
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    /// Convert a single value to a bitmask.
    fn to_mask(val: u8) -> u16 {
        assert!((1..=9).contains(&val), "val must be in 1..=9");
        1 << (val - 1)
    }

    /// Iterator over values available in this set. Note that the iterator is non-borrowing,
    /// because it isn't necessary to keep a borrow for the iterator to work.
    pub(crate) fn iter(&self) -> impl Iterator<Item = u8> + DoubleEndedIterator + FusedIterator {
        let clone = self.clone(); // Cheap u16 copy.
        (1..=9).filter(move |&val| clone.contains(val))
    }
}

impl Default for AvailSet {
    #[inline]
    fn default() -> Self {
        AvailSet::none()
    }
}

/// Like AvailSet but tracks the number of each element available. While AvailSet
/// is useful for a single cell where at most one copy exists, AvailCounter is
/// intended for tracking what's left in entire rows, columns, or sectors, or
/// intersections thereof.
pub(crate) struct AvailCounter(Box<[u8]>);

impl AvailCounter {
    /// Create an AvailCounter with zero of every number.
    pub(crate) fn new() -> Self {
        AvailCounter(vec![0; 9].into_boxed_slice())
    }

    /// Add one of the given number to the counter.
    pub(crate) fn add(&mut self, val: u8) {
        self[val] = self[val].checked_add(1).expect("overflowed counter");
    }

    /// Add all the values from the given set to the counter.
    pub(crate) fn add_all(&mut self, vals: &AvailSet) {
        for val in vals.iter() {
            self.add(val);
        }
    }

    /// Remove one of the given number from the counter.
    pub(crate) fn remove(&mut self, val: u8) {
        self[val] = self[val].checked_sub(1).expect("underflowed counter");
    }

    /// Get the set of available values.
    pub(crate) fn avail(&self) -> AvailSet {
        let mut avail = AvailSet::none();
        for (val, &count) in self.0.iter().enumerate() {
            if count > 0 {
                avail.add(val as u8);
            }
        }
        avail
    }
}

impl Default for AvailCounter {
    fn default() -> Self {
        Self::new()
    }
}

impl Index<u8> for AvailCounter {
    type Output = u8;

    fn index(&self, idx: u8) -> &Self::Output {
        &self.0[idx as usize]
    }
}

impl IndexMut<u8> for AvailCounter {
    fn index_mut(&mut self, idx: u8) -> &mut Self::Output {
        &mut self.0[idx as usize]
    }
}
