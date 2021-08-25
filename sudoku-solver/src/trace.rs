//! Tools for tracing how a solution was reached.
use std::ops::{Index, IndexMut};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::collections::indexed::IndexMap;
use crate::{AvailSet, Board, Col, Coord, Row, Sector, SectorCol, SectorRow, Val};

/// Records steps used during deductive reduction.
pub trait DeductiveTracer {
    /// Record a deduction and the reason why the deduction happened.
    fn deduce(&mut self, reason: DeductionReason, remaining: Remaining);
}

/// Deductive tracer that doesn't record anything.
pub struct NopDeductiveTracer;

impl DeductiveTracer for NopDeductiveTracer {
    fn deduce(&mut self, _: DeductionReason, _: Remaining) {}
}

/// Trace of what was remaining at each coordinate.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(transparent))]
pub struct Remaining(IndexMap<Coord, AvailSet>);

impl Remaining {
    /// Get a Board with only the known remaining values set.
    pub fn board(&self) -> Board {
        let mut board = Board::new();
        for (src, dest) in self.0.as_ref().iter().zip(board.as_mut()) {
            *dest = src.get_single()
        }
        board
    }
}

impl From<IndexMap<Coord, AvailSet>> for Remaining {
    fn from(board: IndexMap<Coord, AvailSet>) -> Self {
        Self(board)
    }
}

impl From<Remaining> for IndexMap<Coord, AvailSet> {
    fn from(rem: Remaining) -> Self {
        rem.0
    }
}

impl Index<Coord> for Remaining {
    type Output = AvailSet;

    fn index(&self, idx: Coord) -> &Self::Output {
        &self.0[idx]
    }
}

impl IndexMut<Coord> for Remaining {
    fn index_mut(&mut self, idx: Coord) -> &mut Self::Output {
        &mut self.0[idx]
    }
}

impl AsRef<[AvailSet]> for Remaining {
    fn as_ref(&self) -> &[AvailSet] {
        self.0.as_ref()
    }
}

impl AsMut<[AvailSet]> for Remaining {
    fn as_mut(&mut self) -> &mut [AvailSet] {
        self.0.as_mut()
    }
}

/// The cause and result of a single deduction.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Deduction {
    #[cfg_attr(feature = "serde", serde(flatten))]
    pub reason: DeductionReason,
    pub remaining: Remaining,
}

/// Reason a deduction could be performed.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "reason"),
    serde(rename_all = "snake_case")
)]
pub enum DeductionReason {
    /// Initial state of the board before deduction.
    InitialState,
    /// The given coordinate had only one value left, so that value could be
    /// eliminated from all neighbors of that coordinate.
    CoordNeighbors { pos: Coord, val: Val },
    /// The given values each had only one cell left in the given row, so any
    /// other values from those positions could be excluded.
    UniqueInRow { pos: Row, vals: AvailSet },
    /// The given values each had only one cell left in the given col, so any
    /// other values from those positions could be excluded.
    UniqueInCol { pos: Col, vals: AvailSet },
    /// The given values each had only one cell left in the given sector, so any
    /// other values from those positions could be excluded.
    UniqueInSector { pos: Sector, vals: AvailSet },
    /// The given sector-row has exactly 3 values left, so those can be
    /// eliminated from the rest of the sector and row. The given values are the
    /// ones that actually changed.
    SecRowTriple { pos: SectorRow, vals: AvailSet },
    /// The given sector-col has exactly 3 values left, so those can be
    /// eliminated from the rest of the sector and col. The given values are the
    /// ones that actually changed.
    SecColTriple { pos: SectorCol, vals: AvailSet },
    /// The given sector-row is the only one in the sector that could hold the
    /// given values, so those values are eliminated from the rest of the row.
    SecOnlyRow { pos: SectorRow, vals: AvailSet },
    /// The given sector-col is the only one in the sector that could hold the
    /// given values, so those values are eliminated from the rest of the column.
    SecOnlyCol { pos: SectorCol, vals: AvailSet },
    /// The given sector-row is the only one left in the row that could hold the
    /// given value, so those values have been eliminated from the rest of the
    /// sector.
    RowOnlySec { pos: SectorRow, vals: AvailSet },
    /// The given sector-col is the only one left in the column that could hold
    /// the given value, so those values have been eliminated from the rest of
    /// the sector.
    ColOnlySec { pos: SectorCol, vals: AvailSet },
    /// The board was proven unsolveable for the given reason.
    Unsolveable(UnsolveableReason),
}

/// Reason the board cannot be solved.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "reason"),
    serde(rename_all = "snake_case")
)]
pub enum UnsolveableReason {
    /// There were no more possible values for the given coordinate.
    Empty { pos: Coord },
    /// In the given row the given values each only had one cell left and it was
    /// the same cell for both.
    RowValsMustShare {
        pos: Row, 
        vals: AvailSet
    },
    /// In the given col the given values each only had one cell left and it was
    /// the same cell for both.
    ColValsMustShare {
        pos: Col, 
        vals: AvailSet
    },
    /// In the given sector the given values each only had one cell left and it
    /// was the same cell for both.
    SecValsMustShare {
        pos: Sector, 
        vals: AvailSet
    },
    /// The last possible position for the given val was eliminated from the row.
    RowMissingVal {
        pos: Row,
        val: Val,
    },
    /// The last possible position for the given val was eliminated from the col.
    ColMissingVal {
        pos: Col,
        val: Val,
    },
    /// The last possible position for the given val was eliminated from the
    /// sector.
    SecMissingVal {
        pos: Sector,
        val: Val,
    },
    SecRowTooFewVals { pos: SectorRow, },
    SecColTooFewVals { pos: SectorCol, },
}
