use std::borrow::Borrow;
use std::hash::{Hash, Hasher};
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::ops::Range;
use std::ops::{Index, IndexMut};

/// Map over over some type that can convert to a flat index. This map does not allow
/// values to be absent; any value not explicitly set will have a default value stored.
/// This will therefore mean that the map always has the size of the number of indexes.
/// Indexes must be contiguous.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IndexMap<K, V> {
    cells: Box<[V]>,
    _key: PhantomData<K>,
}

impl<K, V> IndexMap<K, V>
where
    K: FixedSizeIndex,
    V: Default,
{
    /// Construct an indexed map with default values for each cell.
    #[allow(unused)]
    pub fn new() -> Self {
        let mut data = Vec::with_capacity(K::NUM_INDEXES);
        for _ in 0..K::NUM_INDEXES {
            data.push(Default::default());
        }
        IndexMap {
            cells: data.into_boxed_slice(),
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
        let mut data = Vec::with_capacity(K::NUM_INDEXES);
        for _ in 0..K::NUM_INDEXES - 1 {
            data.push(val.clone());
        }
        if K::NUM_INDEXES > 0 {
            data.push(val);
        }
        IndexMap {
            cells: data.into_boxed_slice(),
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

    /// Iterator over all cells with their corresponding keys.
    #[allow(unused)]
    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (K, &V)> + ExactSizeIterator + DoubleEndedIterator + FusedIterator
    {
        K::values().zip(self.values())
    }

    /// Iterator over all mut cells with their corresponding keys.
    #[allow(unused)]
    pub fn iter_mut(
        &mut self,
    ) -> impl Iterator<Item = (K, &mut V)> + ExactSizeIterator + DoubleEndedIterator + FusedIterator
    {
        K::values().zip(self.values_mut())
    }

    /// Iterator over just the values of the map.
    #[inline]
    pub fn values(&self) -> std::slice::Iter<V> {
        self.cells.iter()
    }

    /// Mutable iterator over just the values of the map.
    #[inline]
    pub fn values_mut(&mut self) -> std::slice::IterMut<V> {
        self.cells.iter_mut()
    }

    /// Iterator over the keys of the map.
    #[inline]
    #[allow(unused)]
    pub fn keys(&self) -> Values<K> {
        K::values()
    }
}

impl<K, V: Hash> Hash for IndexMap<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.cells.hash(state);
    }
}

impl<I, K, V> Index<I> for IndexMap<K, V>
where
    K: FixedSizeIndex,
    I: Borrow<K>,
{
    type Output = V;

    fn index(&self, idx: I) -> &Self::Output {
        &self.cells[idx.borrow().idx()]
    }
}

impl<I, K, V> IndexMut<I> for IndexMap<K, V>
where
    K: FixedSizeIndex,
    I: Borrow<K>,
{
    fn index_mut(&mut self, idx: I) -> &mut Self::Output {
        &mut self.cells[idx.borrow().idx()]
    }
}

impl<K, V> AsRef<[V]> for IndexMap<K, V> {
    #[inline]
    fn as_ref(&self) -> &[V] {
        &*self.cells
    }
}

impl<K, V> AsMut<[V]> for IndexMap<K, V> {
    #[inline]
    fn as_mut(&mut self) -> &mut [V] {
        &mut *self.cells
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

pub struct Values<I> {
    range: Range<usize>,
    _zone: PhantomData<I>,
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
