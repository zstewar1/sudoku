use std::borrow::Borrow;
use std::convert::TryFrom;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::ops::Range;
use std::ops::{Index, IndexMut};

use thiserror::Error;

/// Map over over some type that can convert to a flat index. This map does not allow
/// values to be absent; any value not explicitly set will have a default value stored.
/// This will therefore mean that the map always has the size of the number of indexes.
/// Indexes must be contiguous.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexMap<K, V> {
    data: Box<[V]>,
    _key: PhantomData<K>,
}

impl<K, V> IndexMap<K, V>
where
    K: FixedSizeIndex,
    V: Default,
{
    /// Construct an indexed map with default values for each cell.
    pub fn new() -> Self {
        let mut data = Vec::with_capacity(K::NUM_INDEXES);
        for _ in 0..K::NUM_INDEXES {
            data.push(Default::default());
        }
        IndexMap {
            data: data.into_boxed_slice(),
            _key: PhantomData,
        }
    }
}

impl<K, V> IndexMap<K, V>
where
    K: FixedSizeIndex,
    V: Clone,
{
    /// Construct an indexed map with the given value for each cell.
    pub fn with_value(val: V) -> Self {
        IndexMap {
            data: vec![val; K::NUM_INDEXES].into_boxed_slice(),
            _key: PhantomData,
        }
    }
}

impl<K, V> IndexMap<K, V>
where
    K: FixedSizeIndex,
{
    /// Length of the map.
    pub const LEN: usize = K::NUM_INDEXES;

    /// Iterator over all data with their corresponding keys.
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (K, &V)> + ExactSizeIterator + DoubleEndedIterator + FusedIterator
    {
        K::values().zip(self.values())
    }

    /// Iterator over all mut data with their corresponding keys.
    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (K, &mut V)> + ExactSizeIterator + DoubleEndedIterator + FusedIterator
    {
        K::values().zip(self.values_mut())
    }

    /// Iterator over just the values of the map.
    #[inline]
    pub fn values(&self) -> std::slice::Iter<V> {
        self.data.iter()
    }

    /// Mutable iterator over just the values of the map.
    #[inline]
    pub fn values_mut(&mut self) -> std::slice::IterMut<V> {
        self.data.iter_mut()
    }

    /// Iterator over the keys of the map.
    #[inline]
    #[allow(unused)]
    pub fn keys(&self) -> Values<K> {
        K::values()
    }

    /// Slice split at mut using the key type.
    #[inline]
    pub fn split_at_mut(&mut self, key: K) -> (&mut [V], &mut [V]) {
        self.data.split_at_mut(key.idx())
    }
}

impl<K, V: Hash> Hash for IndexMap<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl<I, K, V> Index<I> for IndexMap<K, V>
where
    K: FixedSizeIndex,
    I: Borrow<K>,
{
    type Output = V;

    fn index(&self, idx: I) -> &Self::Output {
        &self.data[idx.borrow().idx()]
    }
}

impl<I, K, V> IndexMut<I> for IndexMap<K, V>
where
    K: FixedSizeIndex,
    I: Borrow<K>,
{
    fn index_mut(&mut self, idx: I) -> &mut Self::Output {
        &mut self.data[idx.borrow().idx()]
    }
}

impl<K, V> AsRef<[V]> for IndexMap<K, V> {
    #[inline]
    fn as_ref(&self) -> &[V] {
        &*self.data
    }
}

impl<K, V> AsMut<[V]> for IndexMap<K, V> {
    #[inline]
    fn as_mut(&mut self) -> &mut [V] {
        &mut *self.data
    }
}

impl<K: FixedSizeIndex, V> TryFrom<Vec<V>> for IndexMap<K, V> {
    type Error = IncorrectSize<K, V, Vec<V>>;

    fn try_from(vec: Vec<V>) -> Result<Self, Self::Error> {
        if vec.len() == K::NUM_INDEXES {
            Ok(Self {
                data: vec.into_boxed_slice(),
                _key: PhantomData,
            })
        } else {
            Err(IncorrectSize::new(vec))
        }
    }
}

impl<K: FixedSizeIndex, V> TryFrom<Box<[V]>> for IndexMap<K, V> {
    type Error = IncorrectSize<K, V, Box<[V]>>;

    fn try_from(data: Box<[V]>) -> Result<Self, Self::Error> {
        if data.len() == K::NUM_INDEXES {
            Ok(Self {
                data,
                _key: PhantomData,
            })
        } else {
            Err(IncorrectSize::new(data))
        }
    }
}

impl<K, V> From<IndexMap<K, V>> for Vec<V> {
    #[inline]
    fn from(map: IndexMap<K, V>) -> Self {
        map.data.into()
    }
}

impl<K, V> From<IndexMap<K, V>> for Box<[V]> {
    #[inline]
    fn from(map: IndexMap<K, V>) -> Self {
        map.data
    }
}

/// Indicates that a fixed size collection was constructed from a data collection
/// that was the wrong size.
#[derive(Copy, Clone, Error)]
#[error("tried to initialize an indexed collection from a set of {} elements, but it must have size {}", .0.as_ref().len(), K::NUM_INDEXES)]
pub struct IncorrectSize<K: FixedSizeIndex, V, D: AsRef<[V]>>(D, PhantomData<K>, PhantomData<V>);

impl<K: FixedSizeIndex, V, D: AsRef<[V]>> IncorrectSize<K, V, D> {
    /// Create an error of this type with the given data.
    pub(crate) fn new(data: D) -> Self {
        Self(data, PhantomData, PhantomData)
    }

    /// Get back the original collection that was passed in.
    pub fn into_original(self) -> D {
        self.0
    }
}

impl<K: FixedSizeIndex, V, D: AsRef<[V]>> fmt::Debug for IncorrectSize<K, V, D> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // forward implementation to display
        write!(f, "{}", self)
    }
}

/// Enables a unique minimal index for intersection pairs of (Row, Sector) and
/// (Col, Sector).
pub trait FixedSizeIndex {
    /// Number of converted indexs.
    const NUM_INDEXES: usize;

    fn values() -> Values<Self>
    where
        Self: Sized,
    {
        Values {
            range: 0..Self::NUM_INDEXES,
            _zone: PhantomData,
        }
    }

    /// Convert to a flat index.
    fn idx(&self) -> usize;

    /// Convert from a flat index.
    fn from_idx(idx: usize) -> Self;
}

#[derive(Clone, Debug)]
pub struct Values<I> {
    range: Range<usize>,
    _zone: PhantomData<I>,
}

impl<I: FixedSizeIndex> Values<I> {
    /// Gets a copy of the remaining range of indexes.
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
}

impl<I: FixedSizeIndex> Iterator for Values<I> {
    type Item = I;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.range.next().map(|val| I::from_idx(val))
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.range.size_hint()
    }

    #[inline]
    fn nth(&mut self, n: usize) -> Option<Self::Item> {
        self.range.nth(n).map(|val| I::from_idx(val))
    }

    #[inline]
    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<I: FixedSizeIndex> ExactSizeIterator for Values<I> {}

impl<I: FixedSizeIndex> DoubleEndedIterator for Values<I> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.range.next_back().map(|val| I::from_idx(val))
    }

    fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
        self.range.nth_back(n).map(|val| I::from_idx(val))
    }
}

impl<I: FixedSizeIndex> FusedIterator for Values<I> {}

#[cfg(feature = "serde")]
mod serde {
    use std::fmt;
    use std::marker::PhantomData;

    use serde::de::{Error, SeqAccess, Visitor};
    use serde::ser::SerializeTuple;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    use super::{FixedSizeIndex, IndexMap};

    impl<K, V> Serialize for IndexMap<K, V>
    where
        V: Serialize,
    {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            let mut tup = serializer.serialize_tuple(self.data.len())?;
            for elem in self.data.iter() {
                tup.serialize_element(elem)?;
            }
            tup.end()
        }
    }

    impl<'de, K, V> Deserialize<'de> for IndexMap<K, V>
    where
        K: FixedSizeIndex,
        V: Deserialize<'de>,
    {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
        {
            deserializer.deserialize_tuple(K::NUM_INDEXES, IndexMapVisitor(PhantomData))
        }
    }

    struct IndexMapVisitor<K, V>(PhantomData<fn() -> IndexMap<K, V>>);

    impl<'de, K, V> Visitor<'de> for IndexMapVisitor<K, V>
    where
        K: FixedSizeIndex,
        V: Deserialize<'de>,
    {
        type Value = IndexMap<K, V>;

        fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "a sequence of {} values", K::NUM_INDEXES)
        }

        fn visit_seq<S: SeqAccess<'de>>(self, mut seq: S) -> Result<Self::Value, S::Error> {
            let mut data = Vec::with_capacity(K::NUM_INDEXES);
            loop {
                match seq.next_element()? {
                    Some(next) if data.len() < K::NUM_INDEXES => data.push(next),
                    // If we encounter more when we already have K::NUM_INDEXES, error.
                    Some(_) => return Err(S::Error::invalid_length(data.len() + 1, &self)),
                    None if data.len() != K::NUM_INDEXES => {
                        return Err(S::Error::invalid_length(data.len(), &self))
                    }
                    None => break,
                }
            }
            Ok(IndexMap {
                data: data.into_boxed_slice(),
                _key: PhantomData,
            })
        }
    }
}
