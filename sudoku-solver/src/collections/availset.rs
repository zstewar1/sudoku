use std::iter::FusedIterator;
use std::ops::{Add, AddAssign, BitOr, BitOrAssign, Index, IndexMut, Not, Sub, SubAssign};

use crate::collections::indexed::IndexMap;
use crate::{FixedSizeIndex, Val, Values};

/// Set of available numbers.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct AvailSet(u16);

impl AvailSet {
    /// Create a new AvailSet with all values available.
    #[inline]
    pub const fn all() -> Self {
        AvailSet(0x1ff)
    }

    /// Create an AvailSet with no values available.
    #[inline]
    pub const fn none() -> Self {
        AvailSet(0)
    }

    /// Create an AvailSet containing only the given value.
    #[inline]
    pub fn only(val: Val) -> Self {
        AvailSet(AvailSet::to_mask(val))
    }

    /// Returns true if there are no more values available.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Returns true if this set contains a single element.
    #[inline]
    pub fn is_single(&self) -> bool {
        self.len() == 1
    }

    /// If there is only a single entry, returns that entry.
    pub fn get_single(&self) -> Option<Val> {
        if self.is_single() {
            let v = (self.0.trailing_zeros() + 1) as u8;
            Some(unsafe { Val::new_unchecked(v) })
        } else {
            None
        }
    }

    /// Add the given value to the set. Return true if the value was not in the
    /// set previously.
    pub fn add(&mut self, val: Val) -> bool {
        let added = !self.contains(val);
        *self |= val;
        added
    }

    /// Remove the given value from the set. Return true if the value was in the
    /// set previously.
    pub fn remove(&mut self, val: Val) -> bool {
        let had = self.contains(val);
        *self -= val;
        had
    }

    /// Returns true if the set contains the given value.
    pub fn contains(&self, val: Val) -> bool {
        self.0 & Self::to_mask(val) != 0
    }

    /// Counts the number of values in this set.
    #[inline]
    pub fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    /// Remove any value that don't match the given function.
    pub fn retain(&mut self, mut f: impl FnMut(Val) -> bool) {
        for val in self.iter() {
            if !f(val) {
                *self -= val;
            }
        }
    }

    /// Convert a single value to a bitmask.
    fn to_mask(val: Val) -> u16 {
        1 << val.idx()
    }

    /// Iterator over values available in this set. Note that the iterator is non-borrowing,
    /// because it isn't necessary to keep a borrow for the iterator to work.
    pub fn iter(self) -> AvailSetIter {
        self.into_iter()
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

impl BitOr for AvailSet {
    type Output = Self;

    #[inline]
    fn bitor(mut self, rhs: Self) -> Self::Output {
        self |= rhs;
        self
    }
}

impl BitOrAssign for AvailSet {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl Sub for AvailSet {
    type Output = Self;

    #[inline]
    fn sub(mut self, rhs: Self) -> Self::Output {
        self -= rhs;
        self
    }
}

impl SubAssign for AvailSet {
    #[inline]
    fn sub_assign(&mut self, rhs: Self) {
        self.0 &= !rhs.0;
    }
}

impl IntoIterator for AvailSet {
    type Item = Val;
    type IntoIter = AvailSetIter;

    fn into_iter(self) -> Self::IntoIter {
        AvailSetIter {
            // We use range exclusive because it's easier to work with.
            vals: Val::values(),
            set: self,
        }
    }
}

pub struct AvailSetIter {
    vals: Values<Val>,
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
        let range = self.vals.range();
        // Mask that includes only already-visited bits from the low end of the
        // range.
        let low_mask = (1 << range.start) - 1;
        // Mask that includes only those bits up to but not including the high
        // end of the range.
        let high_mask = (1 << range.end) - 1;
        // Cut down to just those bits in the high-end mask and excluding those
        // in the lower-end mask.
        let mask = high_mask & (!low_mask);
        let size = (self.set.0 & mask).count_ones() as usize;

        (size, Some(size))
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl ExactSizeIterator for AvailSetIter {}

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
pub(crate) struct AvailCounter(IndexMap<Val, u8>);

impl AvailCounter {
    /// Create an AvailCounter with zero of every number.
    #[inline]
    pub(crate) fn new() -> Self {
        Self::with_count(0)
    }

    /// Create an AvailCounter with the given value for every number.
    pub(crate) fn with_count(count: u8) -> Self {
        AvailCounter(IndexMap::with_value(count))
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
        let (lower, mut upper) = self.0.split_at_mut(val);
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
    pub(crate) fn counts(
        &self,
    ) -> impl Iterator<Item = (Val, &u8)> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        self.0.iter()
    }

    /// Iterator over the mutable counts of the values.
    #[allow(unused)]
    pub(crate) fn counts_mut(
        &mut self,
    ) -> impl Iterator<Item = (Val, &mut u8)> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        self.0.iter_mut()
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
        &self.0[val]
    }
}

impl IndexMut<Val> for AvailCounter {
    fn index_mut(&mut self, val: Val) -> &mut Self::Output {
        &mut self.0[val]
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
        for (ct, &add) in self.0.values_mut().zip(other.0.values()) {
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
        for (ct, &sub) in self.0.values_mut().zip(other.0.values()) {
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

#[cfg(feature = "serde")]
mod serde {
    use std::fmt;

    use serde::de::{SeqAccess, Visitor};
    use serde::ser::SerializeSeq;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use crate::Val;

    use super::AvailSet;

    impl Serialize for AvailSet {
        fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
            let mut seq = serializer.serialize_seq(Some(self.len()))?;
            for val in self.iter() {
                seq.serialize_element(&val)?;
            }
            seq.end()
        }
    }

    impl<'de> Deserialize<'de> for AvailSet {
        fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
            deserializer.deserialize_seq(AvailSetVisitor)
        } 
    }

    struct AvailSetVisitor;

    impl<'de> Visitor<'de> for AvailSetVisitor {
        type Value = AvailSet;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.write_str("a sequence of values from 1-9")
        }

        fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<Self::Value, S::Error> {
            let mut set = AvailSet::none();
            while let Some(next) = seq.next_element::<Val>()? {
                set |= next;
            }
            Ok(set)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryInto;

    use super::*;

    #[test]
    fn avail_counter_to_avail() {
        let cases = &[
            (
                AvailCounter(vec![0, 1, 0, 3, 4, 5, 0, 0, 1].try_into().unwrap()),
                AvailSet(0b100111010),
            ),
            (
                AvailCounter(vec![1, 9, 3, 8, 4, 1, 2, 5, 9].try_into().unwrap()),
                AvailSet(0x1ff),
            ),
            (
                AvailCounter(vec![0, 0, 0, 0, 0, 0, 0, 0, 0].try_into().unwrap()),
                AvailSet(0),
            ),
        ];
        for (input, expected) in cases {
            let result = input.avail();
            assert_eq!(result, *expected);
        }
    }

    #[test]
    fn availset_iter_size() {
        let mut iter = AvailSet(0b010_010_110).iter();
        assert_eq!(iter.len(), 4);
        assert_eq!(iter.next(), Some(Val::new(2)));
        assert_eq!(iter.len(), 3);
        assert_eq!(iter.next_back(), Some(Val::new(8)));
        assert_eq!(iter.len(), 2);
        assert_eq!(iter.next(), Some(Val::new(3)));
        assert_eq!(iter.len(), 1);
        assert_eq!(iter.next(), Some(Val::new(5)));
        assert_eq!(iter.len(), 0);
        assert_eq!(iter.next(), None);
    }
}
