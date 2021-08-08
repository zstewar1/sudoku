use std::cmp::{Ordering, PartialOrd};
use std::num::NonZeroU8;
use std::ops::RangeInclusive;
use std::ops::{Index, IndexMut};

use log::trace;

pub use coordinates::{Col, Coord, Intersect, Row, Sector, SectorCol, SectorRow, Zone};

use collections::indexed::{FixedSizeIndex, IndexMap};
use solve::remaining::RemainingTracker;

mod collections;
#[macro_use]
mod coordinates;
mod solve;

/// A Sudoku Board value.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Ord)]
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
            impl From<$t> for Val {
                fn from(val: $t) -> Self {
                    assert!(
                        (Self::MIN as $t..=Self::MAX as $t).contains(&val),
                        "value must be in range [1, 9], got {}",
                        val,
                    );
                    unsafe { Val::new_unchecked(val as u8) }
                }
            }
        )*
    };
}

val_fromint!(u8, i8, u16, i16, u32, i32, u64, i64, u128, i128, usize, isize);

/// Sudoku board, with some values optionally specified.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Board(IndexMap<Coord, Option<Val>>);

impl Board {
    /// Total size of the board.
    pub const SIZE: usize = IndexMap::<Coord, Option<Val>>::LEN;

    /// Create a new board with no positions specified.
    pub fn new() -> Self {
        Default::default()
    }

    /// Manually specify the value of a particular position. Used for setup.
    pub fn specify(&mut self, coord: impl Into<Coord>, val: impl Into<Val>) {
        self[coord.into()] = Some(val.into());
    }

    /// Manually clear the value of a particular position. Used for setup.
    pub fn clear(&mut self, coord: impl Into<Coord>) {
        self[coord.into()] = None;
    }

    /// Get the value at a specific coordinate, if known.
    pub fn get(&self, coord: impl Into<Coord>) -> Option<Val> {
        self[coord.into()]
    }

    /// Attempts to solve this board, returning a board containing all solved values, if a
    /// solution is possible. Otherwise returns None.
    pub fn solve(&self) -> Option<Self> {
        let mut stack = vec![(0, RemainingTracker::new(self))];
        while let Some((depth, next)) = stack.pop() {
            trace!("Trying board at depth {}", depth);
            // Apply deductive rules to eliminate what we can and stop this stack-branch
            // if the board is unsolveable.
            if let Some(reduced) = solve::deductive::reduce(next) {
                if reduced.is_solved() {
                    trace!("Board solved");
                    return Some(reduced.to_board());
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
}

impl Default for Board {
    fn default() -> Self {
        Board(IndexMap::new())
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

/// Reference to a particular row.
///
/// This type always exists behind a reference as a slice within a board. Taking
/// the value out of the reference is undefined behavior.
pub struct RowRef(Option<Val>);

impl Index<Row> for Board {
    type Output = RowRef;

    fn index(&self, row: Row) -> &Self::Output {
        let start = Coord::new(row, 0).idx();
        let start: *const _ = &self.0.as_ref()[start];
        unsafe { &*start.cast() }
    }
}

impl IndexMut<Row> for Board {
    fn index_mut(&mut self, row: Row) -> &mut Self::Output {
        let start = Coord::new(row, 0).idx();
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
pub struct ColRef(Option<Val>);

impl Index<Col> for Board {
    type Output = ColRef;

    fn index(&self, col: Col) -> &Self::Output {
        let start = Coord::new(0, col).idx();
        let start: *const _ = &self.0.as_ref()[start];
        unsafe { &*start.cast() }
    }
}

impl IndexMut<Col> for Board {
    fn index_mut(&mut self, col: Col) -> &mut Self::Output {
        let start = Coord::new(0, col).idx();
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
