//! Implements logic for deductively proving what values belong in which cells.
use std::collections::VecDeque;

use log::trace;

use crate::solve::remaining::RemainingTracker;
use crate::trace::{DeductionReason, DeductiveTracer, UnsolveableReason};
use crate::{AvailSet, Col, Coord, Row, Sector, SectorCol, SectorRow, Val, Zone};

pub(crate) fn reduce<T>(remaining: RemainingTracker, tracer: &mut T) -> Option<RemainingTracker>
where
    T: DeductiveTracer,
{
    let mut reducer = DeductiveReducer::new(remaining, tracer);
    reducer.reduce().ok()?;
    Some(reducer.remaining)
}

struct DeductiveReducer<'a, T> {
    remaining: RemainingTracker,
    queue: VecDeque<ReduceStep>,
    tracer: &'a mut T,
}

impl<'a, T: DeductiveTracer> DeductiveReducer<'a, T> {
    /// Construct a reducer and enqueue the initial reduction steps.
    fn new(remaining: RemainingTracker, tracer: &'a mut T) -> Self {
        let queue = build_queue(&remaining);
        DeductiveReducer {
            remaining,
            queue,
            tracer,
        }
    }

    /// Record the current state of the board with the given reason.
    fn deduce(&mut self, reason: DeductionReason) {
        self.tracer.deduce(reason, self.remaining.remaining());
    }

    /// Reduce the given board by applying the reduction rules.
    fn reduce(&mut self) -> Result<(), ()> {
        self.deduce(DeductionReason::InitialState);
        while let Some(next_step) = self.queue.pop_front() {
            match next_step {
                ReduceStep::CoordSingularized(coord) => self.coord_singularized(coord)?,
                ReduceStep::RowValSingularized(row, val) => self.row_val_singularized(row, val)?,
                ReduceStep::ColValSingularized(col, val) => self.col_val_singularized(col, val)?,
                ReduceStep::SecValSingularized(sec, val) => self.sec_val_singularized(sec, val)?,
                ReduceStep::SecRowTripleized(secrow) => self.secrow_tripleized(secrow)?,
                ReduceStep::SecColTripleized(seccol) => self.seccol_tripleized(seccol)?,
                ReduceStep::SecRowValEliminated(secrow, val) => {
                    self.secrow_val_eliminated(secrow, val)?
                }
                ReduceStep::SecColValEliminated(seccol, val) => {
                    self.seccol_val_eliminated(seccol, val)?
                }
            }
        }
        Ok(())
    }

    /// Visit a coordinate that has been singularized.
    fn coord_singularized(&mut self, coord: Coord) -> Result<(), ()> {
        let mut any_eliminated = false;
        // Note: if a different step eliminates the last number from this cell, we have to
        // stop before we get here again.
        let val = self.remaining[coord].get_single().unwrap();
        for neighbor in coord.neighbors() {
            any_eliminated |= self.eliminate(neighbor, val)?;
        }
        if any_eliminated {
            self.deduce(DeductionReason::CoordNeighbors { pos: coord, val });
        }
        Ok(())
    }

    /// Visit a row which now has only one cell left for some value.
    fn row_val_singularized(&mut self, row: Row, val: Val) -> Result<(), ()> {
        // If this fails, we either enqueued a row-value that still had numbers, or we
        // eliminated the last copy of a number from a row but didn't stop.
        debug_assert!(self.remaining[row][val] == 1);
        // Get the other values in the cell that we are singularizing.
        let (coord, other_vals) = row
            .coords()
            .find_map(|coord| {
                let mut cell = self.remaining[coord];
                if cell.remove(val) {
                    Some((coord, cell))
                } else {
                    None
                }
            })
            .unwrap();
        if !self.eliminate_all(coord, other_vals)?.is_empty() {
            self.deduce(DeductionReason::UniqueInRow { pos: coord, val });
        }
        Ok(())
    }

    /// Visit a col which now has only one cell left for some value.
    fn col_val_singularized(&mut self, col: Col, val: Val) -> Result<(), ()> {
        // If this fails, we either enqueued a col-value that still had numbers, or we
        // eliminated the last copy of a number from a col but didn't stop.
        debug_assert!(self.remaining[col][val] == 1);
        // Get the other values in the cell that we are singularizing.
        let (coord, other_vals) = col
            .coords()
            .find_map(|coord| {
                let mut cell = self.remaining[coord];
                if cell.remove(val) {
                    Some((coord, cell))
                } else {
                    None
                }
            })
            .unwrap();
        if !self.eliminate_all(coord, other_vals)?.is_empty() {
            self.deduce(DeductionReason::UniqueInCol { pos: coord, val });
        }
        Ok(())
    }

    /// Visit a sector which now has only one cell left for some value.
    fn sec_val_singularized(&mut self, sec: Sector, val: Val) -> Result<(), ()> {
        // If this fails, we either enqueued a sec-value that still had numbers, or we
        // eliminated the last copy of a number from a col but didn't stop.
        debug_assert!(self.remaining[sec][val] == 1);
        // Get the other values in the cell that we are singularizing.
        let (coord, other_vals) = sec
            .coords()
            .find_map(|coord| {
                let mut cell = self.remaining[coord];
                if cell.remove(val) {
                    Some((coord, cell))
                } else {
                    None
                }
            })
            .unwrap();
        if !self.eliminate_all(coord, other_vals)?.is_empty() {
            self.deduce(DeductionReason::UniqueInSector { pos: coord, val });
        }
        Ok(())
    }

    /// Eliminates all values in this sector-row from the rest of the row and sector.
    fn secrow_tripleized(&mut self, secrow: SectorRow) -> Result<(), ()> {
        let mut eliminated = AvailSet::none();
        let values = self.remaining[secrow].avail();
        // If this fails we became unsolveable but didn't stop.
        debug_assert!(values.len() == SectorRow::SIZE);
        for neighbor in secrow.neighbors() {
            eliminated |= self.eliminate_all(neighbor, values)?;
        }
        if !eliminated.is_empty() {
            self.deduce(DeductionReason::SecOnlyRow {
                pos: secrow,
                vals: eliminated,
            });
        }
        Ok(())
    }

    /// Eliminates all values in this sector-col from the rest of the col and sector.
    fn seccol_tripleized(&mut self, seccol: SectorCol) -> Result<(), ()> {
        let mut eliminated = AvailSet::none();
        let values = self.remaining[seccol].avail();
        // If this fails we became unsolveable but didn't stop.
        debug_assert!(values.len() == SectorCol::SIZE);
        for neighbor in seccol.neighbors() {
            eliminated |= self.eliminate_all(neighbor, values)?;
        }
        if !eliminated.is_empty() {
            self.deduce(DeductionReason::SecOnlyCol {
                pos: seccol,
                vals: eliminated,
            });
        }
        Ok(())
    }

    /// Handle a value from a sector-row being eliminated.
    fn secrow_val_eliminated(&mut self, secrow: SectorRow, val: Val) -> Result<(), ()> {
        // Visit all sector-row neighbors of the affected sector-row.
        for neighbor in secrow.neighbors() {
            if self.remaining[neighbor][val] == self.remaining[neighbor.row()][val] {
                let mut eliminated = AvailSet::none();
                // If the neighbor is the last one in its row containing the
                // given value, eliminate the value from the rest of the
                // neighbor's sector.
                for sec_secrow in neighbor.sector().rows() {
                    if sec_secrow != neighbor {
                        eliminated |= self.eliminate_all(sec_secrow, Some(val))?;
                    }
                }
                if !eliminated.is_empty() {
                    self.deduce(DeductionReason::RowOnlySec {
                        pos: neighbor,
                        val: val,
                    });
                }
            } else if self.remaining[neighbor][val] == self.remaining[neighbor.sector()][val] {
                let mut eliminated = AvailSet::none();
                // Mutually exclusive with above, because if we hit above, we
                // already eliminated the value from the rest of the sector.
                //
                // If the neighbor is the last one in its sector containing the
                // given value, eliminate the value from the rest of the
                // neighbor's row.
                for row_secrow in neighbor.row().sector_rows() {
                    if row_secrow != neighbor {
                        eliminated |= self.eliminate_all(row_secrow, Some(val))?;
                    }
                }
                if !eliminated.is_empty() {
                    self.deduce(DeductionReason::SecOnlyRow {
                        pos: neighbor,
                        vals: AvailSet::only(val),
                    });
                }
            }
        }
        Ok(())
    }

    /// Handle a value from a sector-col being eliminated.
    fn seccol_val_eliminated(&mut self, seccol: SectorCol, val: Val) -> Result<(), ()> {
        // Visit all sector-col neighbors of the affected sector-col.
        for neighbor in seccol.neighbors() {
            if self.remaining[neighbor][val] == self.remaining[neighbor.col()][val] {
                let mut eliminated = AvailSet::none();
                // If the neighbor is the last one in its col containing the
                // given value, eliminate the value from the rest of the
                // neighbor's sector.
                for sec_seccol in neighbor.sector().cols() {
                    if sec_seccol != neighbor {
                        eliminated |= self.eliminate_all(sec_seccol, Some(val))?;
                    }
                }
                if !eliminated.is_empty() {
                    self.deduce(DeductionReason::ColOnlySec {
                        pos: neighbor,
                        val: val,
                    });
                }
            } else if self.remaining[neighbor][val] == self.remaining[neighbor.sector()][val] {
                let mut eliminated = AvailSet::none();
                // Mutually exclusive with above, because if we hit above, we
                // already eliminated the value from the rest of the sector.
                //
                // If the neighbor is the last one in its sector containing the
                // given value, eliminate the value from the rest of the
                // neighbor's row.
                for col_seccol in neighbor.col().sector_cols() {
                    if col_seccol != neighbor {
                        eliminated |= self.eliminate_all(col_seccol, Some(val))?;
                    }
                }
                if !eliminated.is_empty() {
                    self.deduce(DeductionReason::SecOnlyCol {
                        pos: neighbor,
                        vals: eliminated,
                    });
                }
            }
        }
        Ok(())
    }

    /// Convenience function to eliminate all values in the given AvailSet from all coords
    /// in the given zone.
    /// Returns true if any values were eliminated.
    fn eliminate_all(
        &mut self,
        zone: impl IntoIterator<Item = Coord>,
        vals: impl IntoIterator<Item = Val> + Copy,
    ) -> Result<AvailSet, ()> {
        let mut eliminated = AvailSet::none();
        for coord in zone {
            for val in vals {
                if self.eliminate(coord, val)? {
                    eliminated |= val;
                }
            }
        }
        Ok(eliminated)
    }

    /// Eliminate the given value from a single cell, pushing new reduce steps for the
    /// effects on the row, column, an sector.
    fn eliminate(&mut self, coord: Coord, val: Val) -> Result<bool, ()> {
        if self.eliminate_from_cell(coord, val)? {
            self.eliminate_from_row(coord.row(), val)?;
            self.eliminate_from_col(coord.col(), val)?;
            self.eliminate_from_sec(coord.sector(), val)?;
            self.eliminate_from_secrow(coord.sector_row(), val)?;
            self.eliminate_from_seccol(coord.sector_col(), val)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Eliminate a single value only from the board remaining set. Return true
    /// if the cell changed, false otherwase, and Err if this eliminated the last
    /// value from the cell. Pushes reduce instructions for elimination and
    /// singularization of just this cell.
    fn eliminate_from_cell(&mut self, coord: Coord, val: Val) -> Result<bool, ()> {
        let cell = &mut self.remaining[coord];
        if cell.remove(val) {
            // Last value eliminated from the cell.
            if cell.is_empty() {
                self.deduce(DeductionReason::Unsolveable(UnsolveableReason::Empty(
                    coord,
                )));
                trace!(
                    "Stopped deductive because a {:?} had no remaining values",
                    coord
                );
                return Err(());
            }
            if cell.is_single() {
                self.queue.push_back(ReduceStep::CoordSingularized(coord));
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Eliminate a value from a row, pushing a row singularization if needed.
    fn eliminate_from_row(&mut self, row: Row, val: Val) -> Result<(), ()> {
        match self.remaining[row].remove(val) {
            // Last copy of that value eliminated from the row.
            Some(0) => {
                trace!(
                    "Stopped deductive because {:?} no longer had {:?}",
                    row,
                    val
                );
                return Err(());
            }
            Some(1) => self
                .queue
                .push_back(ReduceStep::RowValSingularized(row, val)),
            Some(_) => {}
            None => panic!("Value was previously eliminated but reduction did not stop"),
        }
        Ok(())
    }

    /// Eliminate a value from a col, pushing a col singularization if needed.
    fn eliminate_from_col(&mut self, col: Col, val: Val) -> Result<(), ()> {
        match self.remaining[col].remove(val) {
            // Last copy of that value eliminated from the col.
            Some(0) => {
                trace!(
                    "Stopped deductive because {:?} no longer had {:?}",
                    col,
                    val
                );
                return Err(());
            }
            Some(1) => self
                .queue
                .push_back(ReduceStep::ColValSingularized(col, val)),
            Some(_) => {}
            None => panic!("Value was previously eliminated but reduction did not stop"),
        }
        Ok(())
    }

    /// Eliminate a value from a sector, pushing a sector singularization if needed.
    fn eliminate_from_sec(&mut self, sec: Sector, val: Val) -> Result<(), ()> {
        match self.remaining[sec].remove(val) {
            // Last copy of that value eliminated from the col.
            Some(0) => {
                trace!(
                    "Stopped deductive because {:?} no longer had {:?}",
                    sec,
                    val
                );
                return Err(());
            }
            Some(1) => self
                .queue
                .push_back(ReduceStep::SecValSingularized(sec, val)),
            Some(_) => {}
            None => panic!("Value was previously eliminated but reduction did not stop"),
        }
        Ok(())
    }

    /// Eliminate a value from a sector-row, pushing as needed a sector-row tripleization
    /// or value elimination step.
    fn eliminate_from_secrow(&mut self, secrow: SectorRow, val: Val) -> Result<(), ()> {
        let cell = &mut self.remaining[secrow];
        if let Some(0) = cell.remove(val) {
            // Just eliminated the last of some number from this sector-row, so check if
            // the number of remaining values is equal to or less than the size.
            let num_avail = cell.avail().len();
            if num_avail < SectorRow::SIZE {
                trace!(
                    "Stopped deductive because {:?} had fewer than three values remaining",
                    secrow
                );
                return Err(());
            } else if num_avail == SectorRow::SIZE {
                self.queue.push_back(ReduceStep::SecRowTripleized(secrow));
            }
            self.queue
                .push_back(ReduceStep::SecRowValEliminated(secrow, val));
        }
        Ok(())
    }

    /// Eliminate a value from a sector-column, pushing as needed a sector-column
    /// tripleization or value elimination step.
    fn eliminate_from_seccol(&mut self, seccol: SectorCol, val: Val) -> Result<(), ()> {
        let cell = &mut self.remaining[seccol];
        if let Some(0) = cell.remove(val) {
            // Just eliminated the last of some number from this sector-row, so check if
            // the number of remaining values is equal to or less than the size.
            let num_avail = cell.avail().len();
            if num_avail < SectorCol::SIZE {
                trace!(
                    "Stopped deductive because {:?} had fewer than three values remaining",
                    seccol
                );
                return Err(());
            } else if num_avail == SectorCol::SIZE {
                self.queue.push_back(ReduceStep::SecColTripleized(seccol));
            }
            self.queue
                .push_back(ReduceStep::SecColValEliminated(seccol, val));
        }
        Ok(())
    }
}

/// Steps to apply to reduce the remaining values.
enum ReduceStep {
    /// The given coordinate changed to only have one value left.
    CoordSingularized(Coord),
    /// The given row changed so there is only one slot left that could hold the
    /// given value.
    RowValSingularized(Row, Val),
    /// The given col changed so there is only one slot left that could hold the
    /// given value.
    ColValSingularized(Col, Val),
    /// The given sector changed so there is only one slot left that could hold
    /// the given value.
    SecValSingularized(Sector, Val),
    /// The given sector-row changed so the number of values left is exactly 3.
    SecRowTripleized(SectorRow),
    /// The given sector-col changed so the number of values left is exactly 3.
    SecColTripleized(SectorCol),
    /// The given value was eliminated from this sector-row.
    SecRowValEliminated(SectorRow, Val),
    /// The given value was eliminated from this sector-col.
    SecColValEliminated(SectorCol, Val),
}

/// Find all reduction rules we should start with for the given board.
fn build_queue(remaining: &RemainingTracker) -> VecDeque<ReduceStep> {
    let mut queue = VecDeque::new();
    for (coord, avail) in remaining.board.iter() {
        if avail.is_single() {
            queue.push_back(ReduceStep::CoordSingularized(coord))
        }
    }
    for (row, avail) in remaining.rows.iter() {
        for (val, &count) in avail.counts() {
            if count == 1 {
                queue.push_back(ReduceStep::RowValSingularized(row, val));
            }
        }
    }
    for (col, avail) in remaining.cols.iter() {
        for (val, &count) in avail.counts() {
            if count == 1 {
                queue.push_back(ReduceStep::ColValSingularized(col, val));
            }
        }
    }
    for (sec, avail) in remaining.sectors.iter() {
        for (val, &count) in avail.counts() {
            if count == 1 {
                queue.push_back(ReduceStep::SecValSingularized(sec, val));
            }
        }
    }
    for (secrow, avail) in remaining.sector_rows.iter() {
        if avail.avail().len() == SectorRow::SIZE {
            queue.push_back(ReduceStep::SecRowTripleized(secrow));
        }
        for val in (!avail.avail()).iter() {
            queue.push_back(ReduceStep::SecRowValEliminated(secrow, val));
        }
    }
    for (seccol, avail) in remaining.sector_cols.iter() {
        if avail.avail().len() == SectorCol::SIZE {
            queue.push_back(ReduceStep::SecColTripleized(seccol));
        }
        for val in (!avail.avail()).iter() {
            queue.push_back(ReduceStep::SecColValEliminated(seccol, val));
        }
    }
    queue
}
