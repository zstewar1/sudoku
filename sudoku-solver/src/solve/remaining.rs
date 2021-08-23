use std::ops::{Index, IndexMut};

use log::trace;

use crate::collections::availset::{AvailCounter, AvailSet};
use crate::collections::indexed::IndexMap;
use crate::trace::Remaining;
use crate::{Board, Col, Coord, Row, Sector, SectorCol, SectorRow, Zone};

/// Tracks remaining values in a board.
#[derive(Clone, Debug)]
pub(crate) struct RemainingTracker {
    pub(super) board: IndexMap<Coord, AvailSet>,
    pub(super) rows: IndexMap<Row, AvailCounter>,
    pub(super) cols: IndexMap<Col, AvailCounter>,
    pub(super) sectors: IndexMap<Sector, AvailCounter>,
    pub(super) sector_rows: IndexMap<SectorRow, AvailCounter>,
    pub(super) sector_cols: IndexMap<SectorCol, AvailCounter>,
}

impl RemainingTracker {
    /// Construct a new tracker from the given board.
    pub(crate) fn new(board: &Board) -> Self {
        let mut tracker = RemainingTracker {
            board: IndexMap::with_value(AvailSet::all()),
            rows: IndexMap::with_value(AvailCounter::with_count(Row::SIZE as u8)),
            cols: IndexMap::with_value(AvailCounter::with_count(Col::SIZE as u8)),
            sectors: IndexMap::with_value(AvailCounter::with_count(Sector::SIZE as u8)),
            sector_rows: IndexMap::with_value(AvailCounter::with_count(SectorRow::SIZE as u8)),
            sector_cols: IndexMap::with_value(AvailCounter::with_count(SectorCol::SIZE as u8)),
        };
        for coord in Coord::all() {
            if let Some(val) = board[coord] {
                tracker.board[coord] = AvailSet::only(val);
                tracker.rows[coord.row()].remove_except(val);
                tracker.cols[coord.col()].remove_except(val);
                tracker.sectors[coord.sector()].remove_except(val);
                tracker.sector_rows[coord.sector_row()].remove_except(val);
                tracker.sector_cols[coord.sector_col()].remove_except(val);
            }
        }
        tracker
    }

    // Return true if the board is known to be unsolveable from its current state.
    pub(crate) fn known_unsolveable(&self) -> bool {
        self.board.values().any(|val| val.is_empty())
            || self
                .rows
                .values()
                .any(|vals| vals.avail().len() < Row::SIZE)
            || self
                .cols
                .values()
                .any(|vals| vals.avail().len() < Col::SIZE)
            || self
                .sectors
                .values()
                .any(|vals| vals.avail().len() < Sector::SIZE)
    }

    // Return true if the board is already solved.
    pub(crate) fn is_solved(&self) -> bool {
        self.rows.values().all(is_solved_zone)
            && self.cols.values().all(is_solved_zone)
            && self.sectors.values().all(is_solved_zone)
    }

    /// Construct a board containing the current state of the solution.
    pub(crate) fn into_board(self) -> Board {
        self.into_remaining().board()
    }

    /// Construct a Remaining tracking only the known cell values.
    /// Note that tracking only the cell values is sufficent to losslessly reconstruct the
    /// remaining tracker.
    pub(crate) fn remaining(&self) -> Remaining {
        self.board.clone().into()
    }

    /// Construct a Remaining tracking only the known cell values.
    /// Note that tracking only the cell values is sufficent to losslessly reconstruct the
    /// remaining tracker.
    pub(crate) fn into_remaining(self) -> Remaining {
        self.board.into()
    }

    /// Find the first cell with multiple values and return an iterator over copies of
    /// this board with that cell specified to each of the possible values.
    pub(crate) fn specify_one(self) -> impl Iterator<Item = Self> {
        // If none has multiple values available, we should either be solved or have
        // failed solving.
        let (coord, avail) = self
            .board
            .iter()
            .find(|(_, avail)| avail.len() > 1)
            .map(|(coord, avail)| (coord, *avail))
            .unwrap();
        trace!("Guessing {:?} with values {:?}", coord, avail);
        avail.iter().filter_map(move |val| {
            let mut copy = self.clone();
            let removed_values = avail - val;
            copy[coord] = AvailSet::only(val);
            copy[coord.row()] -= removed_values;
            copy[coord.col()] -= removed_values;
            copy[coord.sector()] -= removed_values;
            copy[coord.sector_row()] -= removed_values;
            copy[coord.sector_col()] -= removed_values;
            if copy.known_unsolveable() {
                trace!("Skipping {:?} because it is known to be unsolveable.", val);
                None
            } else {
                trace!("Adding copy.");
                Some(copy)
            }
        })
    }
}

fn is_solved_zone(avail: &AvailCounter) -> bool {
    avail.counts().all(|(_, &cnt)| cnt == 1)
}

macro_rules! index {
    ($idx:ty, $out:ty, $field:ident) => {
        impl Index<$idx> for RemainingTracker {
            type Output = $out;

            fn index(&self, idx: $idx) -> &Self::Output {
                &self.$field[idx]
            }
        }

        impl IndexMut<$idx> for RemainingTracker {
            fn index_mut(&mut self, idx: $idx) -> &mut Self::Output {
                &mut self.$field[idx]
            }
        }
    };
}

index!(Coord, AvailSet, board);
index!(Row, AvailCounter, rows);
index!(Col, AvailCounter, cols);
index!(Sector, AvailCounter, sectors);
index!(SectorRow, AvailCounter, sector_rows);
index!(SectorCol, AvailCounter, sector_cols);

impl From<RemainingTracker> for Remaining {
    fn from(tracker: RemainingTracker) -> Self {
        tracker.into_remaining()
    }
}