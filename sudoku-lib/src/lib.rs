use std::collections::VecDeque;

pub use coordinates::{Col, Coord, Row, Sector, Zone};

use collections::availset::{AvailSet, AvailCounter};
use collections::indexed::{IndexMap, FixedSizeIndex};

mod collections;
#[macro_use]
mod coordinates;

/// Sudoku Board.
#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Board(IndexMap<Coord, AvailSet>);

impl Board {
    /// Total size of the board.
    pub const SIZE: usize = (Row::SIZE * Col::SIZE) as usize;

    /// Create a new board with no positions specified.
    pub fn new() -> Self {
        Default::default()
    }

    /// Manually specify the value of a particular position. Used for setup.
    pub fn specify(&mut self, coord: impl Into<Coord>, val: u8) {
        *self.cell_mut(coord) = AvailSet::only(val);
    }

    /// Get the value at a specific coordinate, if known.
    pub fn get(&self, pos: impl Into<Coord>) -> Option<u8> {
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

    /// Iterate over all cell coords where the value is known (exactly 1 value left).
    fn known_cells<'a>(&'a self) -> impl 'a + Iterator<Item = (Coord, &AvailSet)> + DoubleEndedIterator {
        self.0.values().enumerate().filter_map(|(idx, v)| if v.is_single() {
            Some((Coord::from_idx(idx), v))
        } else {
            None
        })
    }

    /// Reduce this board by eliminating numbers that are definitely excluded.
    /// Returns false if reduction eliminated all possible numbers from any cell, which means that
    /// this board is unsolveable.
    fn deductive_reduce(&mut self) -> bool {
        // let mut rowsec = IndexMap::<(Row, Sector), AvailCounter>::new();
        // let mut colsec = IndexMap::<(Col, Sector), AvailCounter>::new();

        // for idx in 0..Self::SIZE {
        //     let coord = Coord::from_flat_index(idx);
        //     rowsec[(coord.row(), coord.sector())].add_all(&self.0[idx]);
        //     colsec[(coord.col(), coord.sector())].add_all(&self.0[idx]);
        // }

        let mut queue: VecDeque<_> = self.known_cells().map(|(k, _)| k).collect();
        while let Some(pos) = queue.pop_front() {
            let val = self.get(pos).expect("Should only enqueue singular cells");
            for neighbor in pos.neighbors() {
                let n = self.cell_mut(neighbor);
                // Don't revisit cells that didn't change.
                if n.remove(val) {
                    // If the last entry was removed from the cell, there is no solution from
                    // here, so stop and return false.
                    if n.is_empty() {
                        return false;
                    }

                    // Whenever we successfully remove a value from a cell, also remove 
                    // from the corresponding row/col + sector intersects.
                    // rowsec[(neighbor.row(), neighbor.sector())].remove(val);
                    // colsec[(neighbor.col(), neighbor.sector())].remove(val);

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
        let cell = self.0.values().enumerate()
            .find_map(|(idx, val)| {
                if !val.is_single() {
                    Some(Coord::from_idx(idx))
                } else {
                    None
                }
            })
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

    /// Returns true if the board is solved. Note: this only checks if all
    /// numbers have been singularized, it does not check whether any numbers
    /// conflict. To prevent conflicts, you first need to run deductive_reduce.
    fn is_solved(&self) -> bool {
        self.0.values().all(|val| val.is_single())
    }

    fn cell(&self, pos: impl Into<Coord>) -> &AvailSet {
        &self.0[pos.into()]
    }

    fn cell_mut(&mut self, pos: impl Into<Coord>) -> &mut AvailSet {
        &mut self.0[pos.into()]
    }
}

impl Default for Board {
    fn default() -> Self {
        Board(IndexMap::with_value(AvailSet::all()))
    }
}

impl<T: AsRef<[u8]>> From<T> for Board {
    /// Convenience method for building a board for in a test. Use a single-dimensional vector of
    /// 81 cells. Unlike [`AvailSet::only`] and [`Board::specify`], 0 is accepted as a value, in
    /// order to mark a cell as not having a specified value. This is instead of using `Option<u8>`
    /// because it is more convenient for tests.
    fn from(values: T) -> Self {
        let values = values.as_ref();
        assert!(values.len() == Self::SIZE);
        let mut board = Board::new();
        for (cell, val) in board.0.values_mut().zip(values.iter().copied()) {
            if val != 0 {
                *cell = AvailSet::only(val);
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
