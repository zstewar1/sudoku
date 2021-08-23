use std::cmp::{Ordering, PartialOrd};
use std::convert::{TryFrom, TryInto};
use std::iter::FusedIterator;
use std::num::NonZeroU8;
use std::ops::RangeInclusive;
use std::ops::{Index, IndexMut};

use log::trace;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub use collections::availset::AvailSet;
pub use collections::indexed::{IncorrectSize, Values};
pub use coordinates::{Col, Coord, Intersect, OutOfRange, Row, Sector, SectorCol, SectorRow, Zone};

use collections::indexed::{FixedSizeIndex, IndexMap};
use solve::remaining::RemainingTracker;

mod collections;
#[macro_use]
mod coordinates;
mod solve;
pub mod trace;

/// A Sudoku Board value.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord)]
#[repr(transparent)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(try_from = "u8"),
    serde(into = "u8")
)]
pub struct Val(NonZeroU8);

impl Val {
    /// Minimum allowed value.
    pub const MIN: u8 = 1;
    /// Max allowed value.
    pub const MAX: u8 = 9;

    /// The range of values that are valid as part of the `Board`.
    pub const VALID_RANGE: RangeInclusive<u8> = Self::MIN..=Self::MAX;

    #[inline]
    pub(crate) const unsafe fn new_unchecked(val: u8) -> Self {
        Val(NonZeroU8::new_unchecked(val))
    }

    /// Create a new Val with the given value.
    pub fn new(val: u8) -> Self {
        assert!(
            Self::VALID_RANGE.contains(&val),
            "value must be in range [1, 9], got {}",
            val
        );
        Val(unsafe { NonZeroU8::new_unchecked(val) })
    }

    /// Get the value as a u8.
    #[inline]
    pub const fn val(self) -> u8 {
        self.0.get()
    }
}

impl FixedSizeIndex for Val {
    const NUM_INDEXES: usize = (Self::MAX - Self::MIN + 1) as usize;

    #[inline]
    fn idx(&self) -> usize {
        (self.0.get() - 1) as usize
    }

    #[inline]
    fn from_idx(idx: usize) -> Self {
        assert!(
            (0..Self::NUM_INDEXES).contains(&idx),
            "Val index must be in range [0, {}), got {}",
            Self::NUM_INDEXES,
            idx
        );
        unsafe { Self::new_unchecked(idx as u8 + 1) }
    }
}

impl PartialOrd for Val {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

macro_rules! val_fromint {
    ($($t:ty),*) => {
        $(
            impl std::convert::TryFrom<$t> for Val {
                type Error = OutOfRange<$t>;

                fn try_from(val: $t) -> Result<Self, Self::Error> {
                    if !(Self::MIN as $t..=Self::MAX as $t).contains(&val) {
                        Err(OutOfRange(val))
                    } else {
                        Ok(unsafe { Val::new_unchecked(val as u8) })
                    }
                }
            }

            impl From<Val> for $t {
                fn from(val: Val) -> $t {
                    val.val() as $t
                }
            }
        )*
    };
}

val_fromint!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

/// Sudoku board, with some values optionally specified.
#[derive(Clone, Debug, Default, Eq, PartialEq, Hash)]
#[repr(transparent)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(transparent))]
pub struct Board(IndexMap<Coord, Option<Val>>);

impl Board {
    /// Total size of the board.
    pub const SIZE: usize = IndexMap::<Coord, Option<Val>>::LEN;

    /// Create a new board with no positions specified.
    pub fn new() -> Self {
        Default::default()
    }

    /// Attempts to solve this board, returning a board containing all solved values, if a
    /// solution is possible. Otherwise returns None.
    pub fn solve(&self) -> Option<Self> {
        let mut stack = vec![(0, RemainingTracker::new(self))];
        while let Some((depth, next)) = stack.pop() {
            trace!("Trying board at depth {}", depth);
            let mut tracer = trace::NopDeductiveTracer;
            // Apply deductive rules to eliminate what we can and stop this stack-branch
            // if the board is unsolveable.
            if let Some(reduced) = solve::deductive::reduce(next, &mut tracer) {
                if reduced.is_solved() {
                    trace!("Board solved");
                    return Some(reduced.into_board());
                } else {
                    trace!("Board reduced but not yet solved.");
                    let len = stack.len();
                    for choice in reduced.specify_one() {
                        stack.push((depth + 1, choice));
                    }
                    trace!("Pushed {} boards at depth {}", stack.len() - len, depth + 1);
                }
            } else {
                trace!("Board could not be reduced.");
            }
        }
        trace!("Ran out of boards to try.");
        // No solution found.
        None
    }

    /// Return true if the board is known to be unsolveable.
    pub fn known_unsolveable(&self) -> bool {
        RemainingTracker::new(self).known_unsolveable()
    }

    /// Return true if the board is solved.
    pub fn is_solved(&self) -> bool {
        RemainingTracker::new(self).is_solved()
    }

    /// View of the board as a flat slice in row-major order.
    #[inline]
    pub fn row_major(&self) -> &[Option<Val>] {
        self.0.as_ref()
    }

    /// Mutable view of the board as a flat slice in row-major order.
    #[inline]
    pub fn row_major_mut(&mut self) -> &mut [Option<Val>] {
        self.0.as_mut()
    }

    /// Iterator over const references to the rows of this board.
    pub fn rows(
        &self,
    ) -> impl '_ + Iterator<Item = &RowRef> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        Row::values().map(move |row| &self[row])
    }

    /// Iterator over mut references to the rows of this board.
    pub fn rows_mut(
        &mut self,
    ) -> impl '_ + Iterator<Item = &mut RowRef> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        let mut start: *mut _ = &mut self.0.as_mut()[0];
        (0..Row::NUM_INDEXES).map(move |_| {
            // This is safe because rows won't alias.
            let res = unsafe { &mut *start.cast() };
            start = unsafe { start.add(Row::SIZE) };
            res
        })
    }

    /// Iterator over const references to the cols of this board.
    pub fn cols(
        &self,
    ) -> impl '_ + Iterator<Item = &ColRef> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        Col::values().map(move |col| &self[col])
    }

    /// Iterator over mut references to the rows of this board.
    pub fn cols_mut(
        &mut self,
    ) -> impl '_ + Iterator<Item = &mut RowRef> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        let mut start: *mut _ = &mut self.0.as_mut()[0];
        (0..Col::NUM_INDEXES).map(move |_| {
            // This is safe because we won't alias.
            let res = unsafe { &mut *start.cast() };
            start = unsafe { start.add(1) };
            res
        })
    }
}

impl AsRef<[Option<Val>]> for Board {
    fn as_ref(&self) -> &[Option<Val>] {
        self.row_major()
    }
}

impl AsMut<[Option<Val>]> for Board {
    fn as_mut(&mut self) -> &mut [Option<Val>] {
        self.row_major_mut()
    }
}

impl Index<Coord> for Board {
    type Output = Option<Val>;

    fn index(&self, coord: Coord) -> &Option<Val> {
        &self.0[coord]
    }
}

impl IndexMut<Coord> for Board {
    fn index_mut(&mut self, coord: Coord) -> &mut Option<Val> {
        &mut self.0[coord]
    }
}

impl TryFrom<Vec<Option<Val>>> for Board {
    type Error = IncorrectSize<Coord, Option<Val>, Vec<Option<Val>>>;

    fn try_from(data: Vec<Option<Val>>) -> Result<Self, Self::Error> {
        Ok(Board(data.try_into()?))
    }
}

impl TryFrom<Box<[Option<Val>]>> for Board {
    type Error = IncorrectSize<Coord, Option<Val>, Box<[Option<Val>]>>;

    fn try_from(data: Box<[Option<Val>]>) -> Result<Self, Self::Error> {
        Ok(Board(data.try_into()?))
    }
}

impl From<Board> for Vec<Option<Val>> {
    #[inline]
    fn from(board: Board) -> Self {
        board.0.into()
    }
}

impl From<Board> for Box<[Option<Val>]> {
    #[inline]
    fn from(board: Board) -> Self {
        board.0.into()
    }
}

impl From<Board> for IndexMap<Coord, Option<Val>> {
    fn from(board: Board) -> Self {
        board.0
    }
}

impl From<IndexMap<Coord, Option<Val>>> for Board {
    fn from(vals: IndexMap<Coord, Option<Val>>) -> Self {
        Self(vals)
    }
}

/// Reference to a particular row.
///
/// This type always exists behind a reference as a slice within a board. Taking
/// the value out of the reference is undefined behavior.
// transparent is needed for correctness because the layout of rust types is unspecified to allow
// for optimization.
#[repr(transparent)]
pub struct RowRef(Option<Val>);

impl RowRef {
    /// Iterator over const references to the elements of this row.
    pub fn iter(
        &self,
    ) -> impl '_ + Iterator<Item = &Option<Val>> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        Col::values().map(move |col| &self[col])
    }

    /// Iterator over mut references to the elements of this row.
    pub fn iter_mut(
        &mut self,
    ) -> impl '_
           + Iterator<Item = &mut Option<Val>>
           + DoubleEndedIterator
           + ExactSizeIterator
           + FusedIterator {
        let start: *mut _ = &mut self.0;
        Col::values().map(move |col| {
            let offset = col.idx();
            // This is safe (no aliasing) as long as col is unique for each iteration.
            unsafe { &mut *start.add(offset) }
        })
    }
}

impl Index<Row> for Board {
    type Output = RowRef;

    fn index(&self, row: Row) -> &Self::Output {
        let start = Coord::new(row, Col::new(0)).idx();
        let start: *const _ = &self.0.as_ref()[start];
        unsafe { &*start.cast() }
    }
}

impl IndexMut<Row> for Board {
    fn index_mut(&mut self, row: Row) -> &mut Self::Output {
        let start = Coord::new(row, Col::new(0)).idx();
        let start: *mut _ = &mut self.0.as_mut()[start];
        unsafe { &mut *start.cast() }
    }
}

impl Index<Col> for RowRef {
    type Output = Option<Val>;

    fn index(&self, col: Col) -> &Self::Output {
        let start: *const _ = &self.0;
        let offset = col.idx();
        unsafe { &*start.add(offset) }
    }
}

impl IndexMut<Col> for RowRef {
    fn index_mut(&mut self, col: Col) -> &mut Self::Output {
        let start: *mut _ = &mut self.0;
        let offset = col.idx();
        unsafe { &mut *start.add(offset) }
    }
}

impl PartialEq for RowRef {
    fn eq(&self, other: &Self) -> bool {
        Col::values().all(|col| self[col] == other[col])
    }
}

impl Eq for RowRef {}

/// Reference to a particular row.
///
/// This type always exists behind a reference as a slice within a board. Taking
/// the value out of the reference is undefined behavior.
// transparent is needed for correctness because the layout of rust types is unspecified to allow
// for optimization.
#[repr(transparent)]
pub struct ColRef(Option<Val>);

impl ColRef {
    /// Iterator over const references to the elements of this col.
    pub fn iter(
        &self,
    ) -> impl '_ + Iterator<Item = &Option<Val>> + DoubleEndedIterator + ExactSizeIterator + FusedIterator
    {
        Row::values().map(move |row| &self[row])
    }

    /// Iterator over mut references to the elements of this col.
    pub fn iter_mut(
        &mut self,
    ) -> impl '_
           + Iterator<Item = &mut Option<Val>>
           + DoubleEndedIterator
           + ExactSizeIterator
           + FusedIterator {
        let start: *mut _ = &mut self.0;
        Row::values().map(move |row| {
            let offset = row.idx() * Col::NUM_INDEXES;
            // This is safe (no aliasing) as long as row is unique for each iteration.
            unsafe { &mut *start.add(offset) }
        })
    }
}

impl Index<Col> for Board {
    type Output = ColRef;

    fn index(&self, col: Col) -> &Self::Output {
        let start = Coord::new(Row::new(0), col).idx();
        let start: *const _ = &self.0.as_ref()[start];
        unsafe { &*start.cast() }
    }
}

impl IndexMut<Col> for Board {
    fn index_mut(&mut self, col: Col) -> &mut Self::Output {
        let start = Coord::new(Row::new(0), col).idx();
        let start: *mut _ = &mut self.0.as_mut()[start];
        unsafe { &mut *start.cast() }
    }
}

impl Index<Row> for ColRef {
    type Output = Option<Val>;

    fn index(&self, row: Row) -> &Self::Output {
        let start: *const _ = &self.0;
        let offset = row.idx() * Col::NUM_INDEXES;
        unsafe { &*start.add(offset) }
    }
}

impl IndexMut<Row> for ColRef {
    fn index_mut(&mut self, row: Row) -> &mut Self::Output {
        let start: *mut _ = &mut self.0;
        let offset = row.idx() * Col::NUM_INDEXES;
        unsafe { &mut *start.add(offset) }
    }
}

impl PartialEq for ColRef {
    fn eq(&self, other: &Self) -> bool {
        Row::values().all(|row| self[row] == other[row])
    }
}

impl Eq for ColRef {}

/// Set up for testing -- enables logging.
#[cfg(test)]
pub(crate) fn setup() {
    let _ = env_logger::builder().is_test(true).try_init();
}

#[cfg(test)]
mod tests {
    use super::*;

    impl From<[&str; 11]> for Board {
        fn from(values: [&str; 11]) -> Self {
            Self::from(&values[..])
        }
    }

    impl From<&[&str]> for Board {
        /// Convenience method for building a board for in a test. Use a
        /// single-dimensional slice of 11 rows. 1-9 means that number, '|' must be
        /// used as a column separator, ' ' means no value, and any other character
        /// causes a panic. Each row must have eactly 11 characters (9 numbers + 2 separators).
        /// Rows 3 and 7 must be "---+---+---"
        fn from(rows: &[&str]) -> Self {
            assert!(rows.len() == 11);
            assert!(rows[3] == "---+---+---" && rows[7] == "---+---+---");
            let mut board = Board::new();
            for (r, &row) in
                Row::values().zip(rows[0..3].iter().chain(&rows[4..7]).chain(&rows[8..11]))
            {
                for (c, val) in Col::values().zip(parse_row(row)) {
                    board[Coord::new(r, c)] = val;
                }
            }
            board
        }
    }

    fn parse_row(row: &str) -> impl '_ + Iterator<Item = Option<Val>> {
        let row = row.as_bytes();
        assert!(row.len() == 11);
        assert!(row[3] == b'|' && row[7] == b'|');
        row[0..3]
            .iter()
            .chain(&row[4..7])
            .chain(&row[8..11])
            .map(|ch| match ch {
                b'1'..=b'9' => Some(Val::new(ch - b'0')),
                b' ' => None,
                _ => panic!("unsupported val: {}", ch),
            })
    }

    #[test]
    fn val_indexes() {
        let vals: Vec<_> = (1..=9).map(Val::new).collect();
        let expected: Vec<_> = (0..9).collect();
        let result: Vec<_> = vals.iter().map(|val| val.idx()).collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn vals() {
        let expected: Vec<_> = (1..=9).map(Val::new).collect();
        let result: Vec<_> = Val::values().collect();
        assert_eq!(result, expected);
    }

    #[test]
    fn solve_puzzle1() {
        crate::setup();

        let board = Board::from([
            "   |1  |   ",
            "   | 58|6 1",
            "8 1|36 | 9 ",
            "---+---+---",
            "5  |   |4 3",
            "  3|6 1|8  ",
            "6 4|   |  7",
            "---+---+---",
            " 3 | 84|5 6",
            "1 5|72 |   ",
            "   |  3|   ",
        ]);
        let expected = Board::from([
            "467|192|385",
            "329|458|671",
            "851|367|294",
            "---+---+---",
            "518|279|463",
            "273|641|859",
            "694|835|127",
            "---+---+---",
            "732|984|516",
            "145|726|938",
            "986|513|742",
        ]);
        let res = board.solve();
        assert_eq!(res, Some(expected));
    }

    #[test]
    fn solve_puzzle2() {
        crate::setup();

        let board = Board::from([
            "   |8  | 14",
            "1 6|4  |75 ",
            " 47|53 |   ",
            "---+---+---",
            "9  | 5 | 62",
            "   |7 9|   ",
            "63 | 4 |  5",
            "---+---+---",
            "   | 87|34 ",
            " 14|  5|6 9",
            "89 |  4|   ",
        ]);
        let expected = Board::from([
            "359|876|214",
            "186|492|753",
            "247|531|896",
            "---+---+---",
            "978|153|462",
            "425|769|138",
            "631|248|975",
            "---+---+---",
            "562|987|341",
            "714|325|689",
            "893|614|527",
        ]);
        let res = board.solve();
        assert_eq!(res, Some(expected));
    }

    #[test]
    fn solve_puzzle3() {
        crate::setup();

        let board = Board::from([
            " 49|   |65 ",
            " 5 |8 7|  3",
            "   |46 |   ",
            "---+---+---",
            "27 |   |   ",
            "  4|5 1|8  ",
            "   |   | 32",
            "---+---+---",
            "   | 42|   ",
            "9  |3 6| 2 ",
            " 27|   |31 ",
        ]);
        let expected = Board::from([
            "749|213|658",
            "156|897|243",
            "832|465|971",
            "---+---+---",
            "278|634|195",
            "394|521|867",
            "615|789|432",
            "---+---+---",
            "563|142|789",
            "981|376|524",
            "427|958|316",
        ]);
        let res = board.solve();
        assert_eq!(res, Some(expected));
    }

    #[test]
    fn solve_bad() {
        crate::setup();

        let board = Board::from([
            "349|   |65 ",
            " 5 |8 7|  3",
            "   |46 |   ",
            "---+---+---",
            "27 |   |   ",
            "  4|5 1|8  ",
            "   |   | 32",
            "---+---+---",
            "   | 42|   ",
            "9  |3 6| 2 ",
            " 27|   |31 ",
        ]);
        let res = board.solve();
        assert_eq!(res, None);
    }

    #[test]
    fn solve_empty() {
        crate::setup();

        let res = Board::new().solve();
        assert!(res.is_some());
    }
}
