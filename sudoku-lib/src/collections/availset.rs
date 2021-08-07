use std::iter::FusedIterator;
use std::ops::{Add, AddAssign, BitOr, BitOrAssign, Index, IndexMut, Not, Sub, SubAssign};

use crate::{FixedSizeIndex, Val};

/// Set of available numbers.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
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
    pub(crate) fn only(val: Val) -> Self {
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
    pub(crate) fn get_single(&self) -> Option<Val> {
        if self.is_single() {
            let v = (self.0.trailing_zeros() + 1) as u8;
            Some(unsafe { Val::new_unchecked(v) })
        } else {
            None
        }
    }

    /// Add the given value to the set. Return true if the value was not in the
    /// set previously.
    #[allow(unused)]
    pub(crate) fn add(&mut self, val: Val) -> bool {
        let added = !self.contains(val);
        *self |= val;
        added
    }

    /// Remove the given value from the set. Return true if the value was in the
    /// set previously.
    pub(crate) fn remove(&mut self, val: Val) -> bool {
        let had = self.contains(val);
        *self -= val;
        had
    }

    /// Returns true if the set contains the given value.
    pub(crate) fn contains(&self, val: Val) -> bool {
        self.0 & Self::to_mask(val) != 0
    }

    /// Counts the number of values in this set.
    #[inline]
    pub(crate) fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    /// Convert a single value to a bitmask.
    fn to_mask(val: Val) -> u16 {
        1 << val.idx()
    }

    /// Iterator over values available in this set. Note that the iterator is non-borrowing,
    /// because it isn't necessary to keep a borrow for the iterator to work.
    pub(crate) fn iter(self) -> impl Iterator<Item = Val> + DoubleEndedIterator + FusedIterator {
        self.into_iter()
    }
}

impl Default for AvailSet {
    #[inline]
    fn default() -> Self {
        AvailSet::none()
    }
}

impl BitOr<Val> for AvailSet {
    type Output = Self;

    #[inline]
    fn bitor(mut self, rhs: Val) -> Self::Output {
        self |= rhs;
        self
    }
}

impl BitOrAssign<Val> for AvailSet {
    #[inline]
    fn bitor_assign(&mut self, rhs: Val) {
        self.0 |= Self::to_mask(rhs);
    }
}

impl Sub<Val> for AvailSet {
    type Output = Self;

    #[inline]
    fn sub(mut self, rhs: Val) -> Self::Output {
        self -= rhs;
        self
    }
}

impl SubAssign<Val> for AvailSet {
    #[inline]
    fn sub_assign(&mut self, rhs: Val) {
        self.0 &= !Self::to_mask(rhs);
    }
}

impl Not for AvailSet {
    type Output = Self;

    fn not(self) -> Self::Output {
        // Bit invert the contents and then mask back to All.
        AvailSet((!self.0) & AvailSet::all().0)
    }
}

impl IntoIterator for AvailSet {
    type Item = Val;
    type IntoIter = AvailSetIter;

    fn into_iter(self) -> Self::IntoIter {
        AvailSetIter {
            vals: Val::values(),
            set: self,
        }
    }
}

pub struct AvailSetIter {
    vals: crate::collections::indexed::Values<Val>,
    set: AvailSet,
}

impl Iterator for AvailSetIter {
    type Item = Val;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some(val) = self.vals.next() {
            if self.set.contains(val) {
                return Some(val);
            }
        }
        None
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let (_, max) = self.vals.size_hint();
        (0, max)
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl DoubleEndedIterator for AvailSetIter {
    fn next_back(&mut self) -> Option<Self::Item> {
        while let Some(val) = self.vals.next_back() {
            if self.set.contains(val) {
                return Some(val);
            }
        }
        None
    }
}

impl FusedIterator for AvailSetIter {}

/// Like AvailSet but tracks the number of each element available. While AvailSet
/// is useful for a single cell where at most one copy exists, AvailCounter is
/// intended for tracking what's left in entire rows, columns, or sectors, or
/// intersections thereof.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) struct AvailCounter(Box<[u8]>);

impl AvailCounter {
    /// Create an AvailCounter with zero of every number.
    #[inline]
    pub(crate) fn new() -> Self {
        Self::with_count(0)
    }

    /// Create an AvailCounter with the given value for every number.
    pub(crate) fn with_count(count: u8) -> Self {
        AvailCounter(vec![count; Val::VALID_RANGE.len()].into_boxed_slice())
    }

    /// Add one of the given number to the counter. Return the updated count.
    /// Panics if the counter overflows.
    #[allow(unused)]
    pub(crate) fn add(&mut self, val: Val) -> u8 {
        let count = &mut self[val];
        *count = count.checked_add(1).expect("overflowed counter");
        *count
    }

    /// Add all the values from the given set to the counter.
    #[allow(unused)]
    pub(crate) fn add_all(&mut self, vals: AvailSet) {
        for val in vals {
            self.add(val);
        }
    }

    /// Remove one of the given number from the counter. If the value was already
    /// zero, return `None`. Otherwise return the updated value.
    pub(crate) fn remove(&mut self, val: Val) -> Option<u8> {
        let count = &mut self[val];
        if *count == 0 {
            None
        } else {
            *count -= 1;
            Some(*count)
        }
    }

    /// Remove one of every value except the given value.
    pub(crate) fn remove_except(&mut self, val: Val) {
        let (lower, mut upper) = self.0.split_at_mut(val.idx());
        upper = &mut upper[1..];
        for count in lower.iter_mut().chain(upper.iter_mut()) {
            *count = count.saturating_sub(1);
        }
    }

    /// Get the set of available values.
    pub(crate) fn avail(&self) -> AvailSet {
        let mut avail = AvailSet::none();
        for (val, &count) in self.counts() {
            if count > 0 {
                avail |= val;
            }
        }
        avail
    }

    /// Iterator over the counts of the values.
    pub(crate) fn counts<'a>(
        &'a self,
    ) -> impl 'a + Iterator<Item = (Val, &u8)> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        Val::values().zip(self.0.iter())
    }

    /// Iterator over the mutable counts of the values.
    #[allow(unused)]
    pub(crate) fn counts_mut<'a>(
        &'a mut self,
    ) -> impl 'a
           + Iterator<Item = (Val, &mut u8)>
           + DoubleEndedIterator
           + ExactSizeIterator
           + FusedIterator {
        Val::values().zip(self.0.iter_mut())
    }
}

impl Default for AvailCounter {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Index<Val> for AvailCounter {
    type Output = u8;

    fn index(&self, val: Val) -> &Self::Output {
        &self.0[val.idx()]
    }
}

impl IndexMut<Val> for AvailCounter {
    fn index_mut(&mut self, val: Val) -> &mut Self::Output {
        &mut self.0[val.idx()]
    }
}

impl Add for AvailCounter {
    type Output = Self;

    #[inline]
    fn add(mut self, other: Self) -> Self::Output {
        self += other;
        self
    }
}

impl Add<&AvailCounter> for AvailCounter {
    type Output = Self;

    #[inline]
    fn add(mut self, other: &AvailCounter) -> Self::Output {
        self += other;
        self
    }
}

impl AddAssign for AvailCounter {
    #[inline]
    fn add_assign(&mut self, other: AvailCounter) {
        *self += &other;
    }
}

impl AddAssign<&AvailCounter> for AvailCounter {
    fn add_assign(&mut self, other: &AvailCounter) {
        for (ct, &add) in self.0.iter_mut().zip(other.0.iter()) {
            *ct = ct.checked_add(add).expect("overflowed count");
        }
    }
}

impl Sub for AvailCounter {
    type Output = Self;

    #[inline]
    fn sub(mut self, other: Self) -> Self::Output {
        self -= other;
        self
    }
}

impl Sub<&AvailCounter> for AvailCounter {
    type Output = Self;

    #[inline]
    fn sub(mut self, other: &AvailCounter) -> Self::Output {
        self -= other;
        self
    }
}

impl Sub<AvailSet> for AvailCounter {
    type Output = Self;

    #[inline]
    fn sub(mut self, other: AvailSet) -> Self::Output {
        self -= other;
        self
    }
}

impl SubAssign for AvailCounter {
    #[inline]
    fn sub_assign(&mut self, other: AvailCounter) {
        *self -= &other;
    }
}

impl SubAssign<&AvailCounter> for AvailCounter {
    fn sub_assign(&mut self, other: &AvailCounter) {
        for (ct, &sub) in self.0.iter_mut().zip(other.0.iter()) {
            *ct = ct.saturating_sub(sub)
        }
    }
}

impl SubAssign<AvailSet> for AvailCounter {
    fn sub_assign(&mut self, other: AvailSet) {
        for val in other {
            self.remove(val);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn avail_counter_to_avail() {
        let cases = &[
            (AvailCounter(vec![0, 1, 0, 3, 4, 5, 0, 0, 1].into()), AvailSet(0b100111010)),
            (AvailCounter(vec![1, 9, 3, 8, 4, 1, 2, 5, 9].into()), AvailSet(0x1ff)),
            (AvailCounter(vec![0, 0, 0, 0, 0, 0, 0, 0, 0].into()), AvailSet(0)),
        ];
        for (input, expected) in cases {
            let result = input.avail();
            assert_eq!(result, *expected);
        }
    }
}