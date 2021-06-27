use std::collections::VecDeque;

/// Coordinates on the Sudoku board.
#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct Coord {
    /// Row (y).
    pub row: usize,
    /// Column (x).
    pub col: usize,
}

impl Coord {
    /// Construct a new coordinate. Since this is (row, col), note that it is (y, x).
    pub fn new(row: usize, col: usize) -> Self {
        Coord { row, col }
    }
}

impl From<(usize, usize)> for Coord {
    /// Converts an (y-row, x-col) pair to a Coordinate.
    fn from((row, col): (usize, usize)) -> Self {
        Coord { row, col }
    }
}

/// Set of available numbers.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct AvailSet(u16);

impl AvailSet {
    /// Create a new AvailSet with all values available.
    fn all() -> Self {
        AvailSet(0x1ff)
    }

    /// Create an AvailSet with no values available.
    #[allow(dead_code)]
    fn none() -> Self {
        AvailSet(0)
    }

    /// Create an AvailSet containing only the given value.
    fn only(val: u8) -> Self {
        AvailSet(AvailSet::to_mask(val))
    }

    /// Returns true if there are no more values available.
    fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Returns true if this set contains a single element.
    fn is_single(&self) -> bool {
        self.0.count_ones() == 1
    }

    /// If there is only a single entry, returns that entry.
    fn get_single(&self) -> Option<u8> {
        if self.is_single() {
            Some((self.0.trailing_zeros() + 1) as u8)
        } else {
            None
        }
    }

    /// Remove the given value from the set. Return true if the value was in the set previously.
    fn remove(&mut self, val: u8) -> bool {
        let had = self.contains(val);
        self.0 &= !Self::to_mask(val);
        had
    }

    /// Returns true if the set contains the given value.
    fn contains(&self, val: u8) -> bool {
        self.0 & Self::to_mask(val) != 0
    }

    /// Counts the number of values in this set.
    #[allow(dead_code)]
    fn len(&self) -> usize {
        self.0.count_ones() as usize
    }

    /// Convert a single value to a bitmask.
    fn to_mask(val: u8) -> u16 {
        assert!((1..=9).contains(&val), "val must be in 1..=9");
        1 << (val - 1)
    }

    /// Iterator over values available in this set. Note that the iterator is non-borrowing,
    /// because it isn't necessary to keep a borrow for the iterator to work.
    fn iter(&self) -> impl Iterator<Item = u8> {
        let clone = self.clone(); // Cheap u16 copy.
        (1..=9).filter(move |&val| clone.contains(val))
    }
}

/// Sudoku Board.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Board(Vec<AvailSet>);

impl Board {
    pub const WIDTH: usize = 9;
    pub const HEIGHT: usize = 9;

    /// Create a new board with no positions specified.
    pub fn new() -> Self {
        Default::default()
    }

    /// Manually specify the value of a particular position. Used for setup.
    pub fn specify(&mut self, coord: impl Into<Coord>, val: u8) {
        *self.cell_mut(coord) = AvailSet::only(val);
    }

    /// Get the value at a specific coordinate, if known.
    pub fn get(&self, pos: Coord) -> Option<u8> {
        self.cell(pos).get_single()
    }

    /// Consumes this board, returning a board with all positions known, if possible. If the board
    /// cannot be solved, returns None.
    pub fn solve(mut self) -> Option<Board> {
        if !self.deductive_reduce() {
            // If we cannot reduce the starting board, there must be conflicting numbers.
            return None;
        }
        // deductive_reduce ensures there are not duplicated or conflicting numbers.
        if self.is_solved() {
            return Some(self);
        }
        // Because the inductive step always tries all values for the first empty cell, we don't have
        // to worry about re-visiting the same possible solutions ever.
        let mut stack = Vec::new();
        stack.push(self);
        while let Some(next) = stack.pop() {
            for child in next.inductive_reduce() {
                // inductive_reduce runs deductive_reduce and only returns possible steps towards
                // the solution, so is_solved here is safe.
                if child.is_solved() {
                    return Some(child);
                }
                stack.push(child);
            }
        }
        // No solution found.
        None
    }

    /// Iterate over all cell indexes.
    pub fn all_cells() -> impl Iterator<Item = Coord> {
        (0..9).flat_map(|row| (0..9).map(move |col| Coord::new(row, col)))
    }

    /// Iterate over all cell indexes in the given row.
    pub fn row(row: usize) -> impl Iterator<Item = Coord> {
        assert!(row < Self::HEIGHT);
        (0..9).map(move |col| Coord::new(row, col))
    }

    /// Iterate over all cell indexes in the given row.
    pub fn col(col: usize) -> impl Iterator<Item = Coord> {
        assert!(col < Self::WIDTH);
        (0..9).map(move |row| Coord::new(row, col))
    }

    /// Iterate over all cell indexes in the same sector as the given coordinates.
    pub fn sector(pos: Coord) -> impl Iterator<Item = Coord> {
        assert!(pos.row < Self::HEIGHT);
        assert!(pos.col < Self::WIDTH);
        let base_row = pos.row - (pos.row % 3);
        let base_col = pos.col - (pos.col % 3);
        (base_row..)
            .take(3)
            .flat_map(move |row| (base_col..).take(3).map(move |col| Coord::new(row, col)))
    }

    /// Iterate over all cell indexes where the value is known (exactly 1 value left).
    fn known_cells<'a>(&'a self) -> impl 'a + Iterator<Item = Coord> {
        Self::all_cells().filter(move |&cell| self.get(cell).is_some())
    }

    /// Reduce this board by eliminating numbers that are definitely excluded.
    /// Returns false if reduction eliminated all possible numbers from any cell, which means that
    /// this board is unsolveable.
    fn deductive_reduce(&mut self) -> bool {
        let mut queue: VecDeque<_> = self.known_cells().collect();
        while let Some(pos) = queue.pop_front() {
            let val = self.get(pos).expect("Should only enqueue singular cells");
            for neighbor in Self::neighbors(pos) {
                let n = self.cell_mut(neighbor);
                // Don't revisit cells that didn't change.
                if n.remove(val) {
                    // If the last entry was removed from the cell, there is no solution from
                    // here, so stop and return false.
                    if n.is_empty() {
                        return false;
                    }
                    // If the neighbor has been reduced to having a single value left, then we
                    // may be able to eliminate more values by visiting it again
                    if n.is_single() && !queue.contains(&neighbor) {
                        queue.push_back(neighbor);
                    }
                }
            }
        }
        true
    }

    /// Inductively reduce the board by finding the fist cell that isn't fully specified and
    /// returning copies of the board with every possible solution for that cell.
    fn inductive_reduce<'a>(&'a self) -> impl 'a + Iterator<Item = Board> {
        let cell = Self::all_cells()
            .find(|&cell| !self.cell(cell).is_single())
            .expect("Board is already solved or has cells with no remaining values");
        let choices = self.cell(cell).iter();
        choices.filter_map(move |val| {
            let mut board = self.clone();
            *board.cell_mut(cell) = AvailSet::only(val);
            if board.deductive_reduce() {
                Some(board)
            } else {
                None
            }
        })
    }

    /// Returns true if the board is solved. Note: this only checks if all numbers have been
    /// singularized, it does not check whether any numbers conflict. To prevent conflicts, you
    /// first need to run deductive_reduce.
    fn is_solved(&self) -> bool {
        for cell in Self::all_cells() {
            if !self.cell(cell).is_single() {
                return false;
            }
        }
        true
    }

    /// Gets the indexes of all neighbors of the given coordinate (tiles in the same row, column,
    /// or sector.
    fn neighbors(pos: Coord) -> impl Iterator<Item = Coord> {
        Self::row(pos.row)
            .chain(Self::col(pos.col))
            .chain(
                Self::sector(pos)
                    // others with the same row or col are already covered by the previous two
                    // iterators, don't visit them twice.
                    .filter(move |other| other.row != pos.row && other.col != pos.col),
            )
            .filter(move |&other| other != pos)
    }

    fn cell(&self, pos: impl Into<Coord>) -> &AvailSet {
        let pos = pos.into();
        assert!(pos.col < Self::WIDTH);
        assert!(pos.row < Self::HEIGHT);
        &self.0[pos.row * Self::WIDTH + pos.col]
    }

    fn cell_mut(&mut self, pos: impl Into<Coord>) -> &mut AvailSet {
        let pos = pos.into();
        assert!(pos.col < Self::WIDTH);
        assert!(pos.row < Self::HEIGHT);
        &mut self.0[pos.row * Self::WIDTH + pos.col]
    }
}

impl Default for Board {
    fn default() -> Self {
        Board(vec![AvailSet::all(); 81])
    }
}

impl<T: AsRef<[u8]>> From<T> for Board {
    /// Convenience method for building a board for in a test. Use a single-dimensional vector of
    /// 81 cells. Unlike [`AvailSet::only`] and [`Board::specify`], 0 is accepted as a value, in
    /// order to mark a cell as not having a specified value. This is instead of using `Option<u8>`
    /// because it is more convenient for tests.
    fn from(values: T) -> Self {
        let values = values.as_ref();
        assert!(values.len() == 81);
        let mut board = Board::new();
        for (cell, val) in Board::all_cells().zip(values.iter().copied()) {
            if val != 0 {
                board.specify(cell, val);
            }
        }
        board
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[rustfmt::skip]
    fn test_solve() {
        let board = Board::from([
            0,0,0, 1,0,0, 0,0,0,
            0,0,0, 0,5,8, 6,0,1,
            8,0,1, 3,6,0, 0,9,0,

            5,0,0, 0,0,0, 4,0,3,
            0,0,3, 6,0,1, 8,0,0,
            6,0,4, 0,0,0, 0,0,7,

            0,3,0, 0,8,4, 5,0,6,
            1,0,5, 7,2,0, 0,0,0,
            0,0,0, 0,0,3, 0,0,0,
        ]);
        let expected = Board::from([
            4,6,7, 1,9,2, 3,8,5,
            3,2,9, 4,5,8, 6,7,1,
            8,5,1, 3,6,7, 2,9,4,

            5,1,8, 2,7,9, 4,6,3,
            2,7,3, 6,4,1, 8,5,9,
            6,9,4, 8,3,5, 1,2,7,

            7,3,2, 9,8,4, 5,1,6,
            1,4,5, 7,2,6, 9,3,8,
            9,8,6, 5,1,3, 7,4,2,
        ]);
        let res = board.solve();
        assert_eq!(res, Some(expected));
    }
}
