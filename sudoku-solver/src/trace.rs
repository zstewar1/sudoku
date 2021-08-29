//! Tools for tracing how a solution was reached.
use std::ops::{Index, IndexMut};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::collections::indexed::IndexMap;
use crate::{AvailSet, Board, Col, Coord, Row, Sector, SectorCol, SectorRow, Val};

/// Records steps used during solving as a tree of puzzles.
pub trait Tracer {
    /// Type of tracer used for deductive steps.
    type Deductive: DeductiveTracer;

    /// Get a deductive tracer.
    fn deductive_tracer() -> Self::Deductive;

    /// Construct a trace node for a solution. This node may be be added to a
    /// parent but will not have children added to it.
    fn solution(deduction: Self::Deductive) -> Self;

    /// Construct a trace node for a deduction that proved unsolveable. This node
    /// may be be added to a parent but will not have children added to it.
    fn unsolveable(deduction: Self::Deductive) -> Self;

    /// Construct an incomplete guess node. As guesses are attempted, they will
    /// be added to the node with add_child.
    fn guess(deduction: Self::Deductive) -> Self;

    /// Add a child to this node.
    fn add_child(&mut self, child: Self);
}

/// Tracer that doesn't record anything.
#[derive(Copy, Clone, Debug, Default)]
pub struct NopTracer;

impl Tracer for NopTracer {
    type Deductive = NopDeductiveTracer;

    fn deductive_tracer() -> Self::Deductive {
        NopDeductiveTracer
    }

    fn solution(_: Self::Deductive) -> Self {
        Self
    }

    fn unsolveable(_: Self::Deductive) -> Self {
        Self
    }

    fn guess(_: Self::Deductive) -> Self {
        Self
    }

    fn add_child(&mut self, _: Self) {}
}

/// Tracer that records the entire search tree.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(tag = "type"),
    serde(rename_all = "snake_case")
)]
pub enum TraceTree {
    /// A tree node that reached a solution. The last board in the list is
    /// solved.
    Solution { deduction: Vec<Deduction> },
    /// A tree node that proved unsolveable. The last board in the list shows
    /// why.
    Unsolveable { deduction: Vec<Deduction> },
    /// A tree node that required guesses to try to reach a solution.
    Guess {
        /// The deduction leading to the starting point for guesses.
        deduction: Vec<Deduction>,
        /// The guesses tried.
        guesses: Vec<TraceTree>,
    },
}

impl Tracer for TraceTree {
    type Deductive = Vec<Deduction>;

    fn deductive_tracer() -> Self::Deductive {
        Default::default()
    }

    fn solution(deduction: Self::Deductive) -> Self {
        TraceTree::Solution { deduction }
    }

    fn unsolveable(deduction: Self::Deductive) -> Self {
        TraceTree::Unsolveable { deduction }
    }

    fn guess(deduction: Self::Deductive) -> Self {
        TraceTree::Guess {
            deduction,
            guesses: Vec::new(),
        }
    }

    fn add_child(&mut self, child: Self) {
        match self {
            TraceTree::Solution { .. } => panic!("cannot add children to solution nodes"),
            TraceTree::Unsolveable { .. } => panic!("cannot add children to unsolveable nodes"),
            TraceTree::Guess {
                ref mut guesses, ..
            } => guesses.push(child),
        }
    }
}

/// Records steps used during deductive reduction.
pub trait DeductiveTracer {
    /// Record a deduction and the reason why the deduction happened.
    fn deduce(&mut self, reason: DeductionReason, remaining: Remaining);
}

/// Deductive tracer that doesn't record anything.
#[derive(Copy, Clone, Debug, Default)]
pub struct NopDeductiveTracer;

impl DeductiveTracer for NopDeductiveTracer {
    fn deduce(&mut self, _: DeductionReason, _: Remaining) {}
}

impl DeductiveTracer for Vec<Deduction> {
    fn deduce(&mut self, reason: DeductionReason, remaining: Remaining) {
        self.push(Deduction { reason, remaining });
    }
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
    serde(tag = "kind"),
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
    RowValsMustShare { pos: Row, vals: AvailSet },
    /// In the given col the given values each only had one cell left and it was
    /// the same cell for both.
    ColValsMustShare { pos: Col, vals: AvailSet },
    /// In the given sector the given values each only had one cell left and it
    /// was the same cell for both.
    SecValsMustShare { pos: Sector, vals: AvailSet },
    /// The last possible position for the given val was eliminated from the row.
    RowMissingVal { pos: Row, val: Val },
    /// The last possible position for the given val was eliminated from the col.
    ColMissingVal { pos: Col, val: Val },
    /// The last possible position for the given val was eliminated from the
    /// sector.
    SecMissingVal { pos: Sector, val: Val },
    /// Too many values were eliminated from the sector-row.
    SecRowTooFewVals { pos: SectorRow },
    /// Too many values were eliminated from the sector-col.
    SecColTooFewVals { pos: SectorCol },
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "serde")]
    mod serde {
        use super::super::*;

        use log::debug;

        // Note: these tests assert round-tripping support, but are also for
        // printing the serialized json using debug!.
        // Run with:
        // `RUST_LOG=debug cargo test --features serde -- --nocapture`
        // to see the output.

        #[test]
        fn serialize_deduction() {
            crate::setup();

            let deduction = Deduction {
                reason: DeductionReason::CoordNeighbors {
                    pos: Coord::new(Row::new(3), Col::new(5)),
                    val: Val::new(8),
                },
                remaining: IndexMap::with_value(AvailSet::all()).into(),
            };
            let ser = serde_json::to_string(&deduction).unwrap();
            debug!("Deduction CoordNeighbors Ser: {}", ser);
            let roundtrip: Deduction = serde_json::from_str(&ser).unwrap();
            assert_eq!(roundtrip, deduction);
        }

        #[test]
        fn serialize_unsolveable() {
            crate::setup();

            let deduction = Deduction {
                reason: DeductionReason::Unsolveable(UnsolveableReason::Empty {
                    pos: Coord::new(Row::new(3), Col::new(5)),
                }),
                remaining: IndexMap::with_value(AvailSet::none()).into(),
            };

            let ser = serde_json::to_string(&deduction).unwrap();
            debug!("Deduction Unsolveable Ser: {}", ser);
            let roundtrip: Deduction = serde_json::from_str(&ser).unwrap();
            assert_eq!(roundtrip, deduction);
        }

        #[test]
        fn serialize_tree() {
            crate::setup();

            let tree = TraceTree::Solution {
                deduction: vec![Deduction {
                    reason: DeductionReason::CoordNeighbors {
                        pos: Coord::new(Row::new(3), Col::new(5)),
                        val: Val::new(8),
                    },
                    remaining: IndexMap::with_value(AvailSet::all()).into(),
                }],
            };
            let ser = serde_json::to_string(&tree).unwrap();
            debug!("Solution Tree Ser: {}", ser);
            let roundtrip: TraceTree = serde_json::from_str(&ser).unwrap();
            assert_eq!(roundtrip, tree);
        }
    }
}
