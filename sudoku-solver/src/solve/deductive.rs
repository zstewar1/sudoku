//! Implements logic for deductively proving what values belong in which cells.
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashSet};
use std::{array, fmt};

use log::trace;

use crate::collections::availset::AvailCounter;
use crate::solve::remaining::RemainingTracker;
use crate::trace::{DeductionReason, DeductiveTracer, UnsolveableReason};
use crate::{AvailSet, Col, Coord, Row, Sector, SectorCol, SectorRow, Val, Zone};

use super::remaining::ExtractRem;

pub(crate) fn reduce<T>(remaining: RemainingTracker, tracer: T) -> (Option<RemainingTracker>, T)
where
    T: DeductiveTracer,
{
    let mut reducer = DeductiveReducer::new(remaining, tracer);
    match reducer.reduce() {
        Ok(()) => (Some(reducer.remaining), reducer.tracer),
        Err(()) => (None, reducer.tracer),
    }
}

struct DeductiveReducer<T> {
    remaining: RemainingTracker,
    queue: ReduceQueue,
    tracer: T,
}

impl<'a, T: DeductiveTracer> DeductiveReducer<T> {
    /// Construct a reducer and enqueue the initial reduction steps.
    fn new(remaining: RemainingTracker, tracer: T) -> Self {
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

    /// Record the current state of the board with the given reason.
    fn fail(&mut self, reason: UnsolveableReason) {
        self.tracer.deduce(
            DeductionReason::Unsolveable(reason),
            self.remaining.remaining(),
        );
    }

    /// Reduce the given board by applying the reduction rules.
    fn reduce(&mut self) -> Result<(), ()> {
        self.deduce(DeductionReason::InitialState);
        while let Some(next_step) = self.queue.pop() {
            match next_step {
                ReduceStep::CoordSingularized(coord) => self.coord_singularized(coord)?,
                ReduceStep::RowValsSingularized(row) => self.rcs_vals_singularized(row)?,
                ReduceStep::ColValsSingularized(col) => self.rcs_vals_singularized(col)?,
                ReduceStep::SecValsSingularized(sec) => self.rcs_vals_singularized(sec)?,
                ReduceStep::SecRowTripleized(secrow) => self.secrow_seccol_tripleized(secrow)?,
                ReduceStep::SecColTripleized(seccol) => self.secrow_seccol_tripleized(seccol)?,
                ReduceStep::RowOnlySec(secrow) => self.secrow_seccol_only_in_line(secrow)?,
                ReduceStep::SecOnlyRow(secrow) => self.secrow_seccol_only_in_sec(secrow)?,
                ReduceStep::ColOnlySec(seccol) => self.secrow_seccol_only_in_line(seccol)?,
                ReduceStep::SecOnlyCol(seccol) => self.secrow_seccol_only_in_sec(seccol)?,
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
    fn rcs_vals_singularized<Z: RowColSec>(&mut self, rcs: Z) -> Result<(), ()> {
        let singles =
            self.remaining[rcs]
                .counts()
                .fold(AvailSet::none(), |mut singles, (val, &count)| {
                    if count == 1 {
                        singles |= val;
                    }
                    singles
                });
        let mut deduced = AvailSet::none();
        for coord in rcs.coords() {
            let rem = self.remaining[coord];
            let matches = rem & singles;
            if !matches.is_empty() {
                if !matches.is_single() {
                    trace!(
                        "Stopped deductive because {:?} had two values {:?} with {:?} as their only possible position",
                        rcs,
                        matches,
                        coord,
                    );
                    self.fail(rcs.fail_must_share(matches));
                    return Err(());
                }
                let others = rem - singles;
                if !self.eliminate_all(coord, others)?.is_empty() {
                    deduced |= matches;
                }
            }
        }
        if !deduced.is_empty() {
            self.deduce(rcs.deduced(deduced));
        }
        Ok(())
    }

    /// Eliminates all values in this sector-row from the rest of the row and sector.
    fn secrow_seccol_tripleized<Z: SecRowSecCol>(&mut self, srsc: Z) -> Result<(), ()> {
        let values = self.remaining[srsc].avail();
        // If this fails we became unsolveable but didn't stop.
        debug_assert!(values.len() == Z::SIZE);
        let eliminated = self.eliminate_all(
            srsc.line_neighbors().chain(srsc.sec_neighbors()).flatten(),
            values,
        )?;
        if !eliminated.is_empty() {
            self.deduce(srsc.deduced_size_match(eliminated));
        }
        Ok(())
    }

    /// Eliminates values in this sector-row/sector-col which have the same count
    /// as the row/col from the rest of the sector.
    fn secrow_seccol_only_in_line<Z: SecRowSecCol>(&mut self, srsc: Z) -> Result<(), ()> {
        let uniques =
            self.remaining[srsc]
                .counts()
                .fold(AvailSet::none(), |mut uniques, (val, &count)| {
                    if count == self.remaining[srsc.line()][val] {
                        uniques |= val;
                    }
                    uniques
                });
        let deduced = self.eliminate_all(srsc.sec_neighbors().flatten(), uniques)?;
        if !deduced.is_empty() {
            self.deduce(srsc.deduced_only_in_line(deduced));
        }
        Ok(())
    }

    /// Eliminates values in this sector-row/sector-col which have the same count
    /// as the sector from the rest of the row/col.
    fn secrow_seccol_only_in_sec<Z: SecRowSecCol>(&mut self, srsc: Z) -> Result<(), ()> {
        let uniques =
            self.remaining[srsc]
                .counts()
                .fold(AvailSet::none(), |mut uniques, (val, &count)| {
                    if count == self.remaining[srsc.sector()][val] {
                        uniques |= val;
                    }
                    uniques
                });
        let deduced = self.eliminate_all(srsc.line_neighbors().flatten(), uniques)?;
        if !deduced.is_empty() {
            self.deduce(srsc.deduced_only_in_sec(deduced));
        }
        Ok(())
    }

    /// Convenience function to eliminate all values in the given AvailSet from all coords
    /// in the given zone.
    /// Returns the set of values that were eliminated.
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

    /// Eliminate the given value from a single cell, pushing new reduce steps
    /// for the effects on the row, column, an sector. Return true if the value
    /// existed previously.
    fn eliminate(&mut self, coord: Coord, val: Val) -> Result<bool, ()> {
        if self.eliminate_from_cell(coord, val)? {
            self.eliminate_from_rcs(coord.row(), val)?;
            self.eliminate_from_rcs(coord.col(), val)?;
            self.eliminate_from_rcs(coord.sector(), val)?;
            self.eliminate_from_secrow_seccol(coord.sector_row(), val)?;
            self.eliminate_from_secrow_seccol(coord.sector_col(), val)?;
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
                trace!(
                    "Stopped deductive because a {:?} had no remaining values",
                    coord
                );
                self.fail(UnsolveableReason::Empty { pos: coord });
                return Err(());
            }
            if cell.is_single() {
                self.queue.push(ReduceStep::CoordSingularized(coord));
            }
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Eliminate a value from a row, pushing a row singularization if needed.
    fn eliminate_from_rcs<Z: RowColSec>(&mut self, rcs: Z, val: Val) -> Result<(), ()> {
        match self.remaining[rcs].remove(val) {
            // Last copy of that value eliminated from the row.
            Some(0) => {
                trace!(
                    "Stopped deductive because {:?} no longer had {:?}",
                    rcs,
                    val
                );
                self.fail(rcs.fail_missing_val(val));
                return Err(());
            }
            Some(1) => self.queue.push(rcs.visit()),
            Some(_) => {}
            None => panic!("Value was previously eliminated but reduction did not stop"),
        }
        Ok(())
    }

    /// Eliminate a value from a sector-row/sector-col, pushing as needed a sector-row
    /// tripleization or value elimination step.
    fn eliminate_from_secrow_seccol<Z: SecRowSecCol>(
        &mut self,
        srsc: Z,
        val: Val,
    ) -> Result<(), ()> {
        let cell = &mut self.remaining[srsc];
        if let Some(0) = cell.remove(val) {
            // Just eliminated the last of some number from this sector-row, so check if
            // the number of remaining values is equal to or less than the size.
            let num_avail = cell.avail().len();
            if num_avail < Z::SIZE {
                trace!(
                    "Stopped deductive because {:?} had fewer than {} values remaining",
                    srsc,
                    Z::SIZE,
                );
                self.fail(srsc.fail_too_few_vals());
                return Err(());
            } else if num_avail == Z::SIZE {
                self.queue.push(srsc.visit_size_match());
            }
            for neighbor in srsc.line_neighbors() {
                if self.remaining[neighbor][val] == self.remaining[neighbor.line()][val] {
                    self.queue.push(neighbor.visit_only_in_line());
                    break;
                }
            }
            for neighbor in srsc.sec_neighbors() {
                if self.remaining[neighbor][val] == self.remaining[neighbor.sector()][val] {
                    self.queue.push(neighbor.visit_only_in_sec());
                    break;
                }
            }
        }
        Ok(())
    }
}

/// Helper for generalizing row/col/sector.
trait RowColSec: Zone + fmt::Debug + Copy + ExtractRem<Avail = AvailCounter> {
    /// Build a reduce step to visit this.
    fn visit(self) -> ReduceStep;

    /// Build the DeductionReason for this.
    fn deduced(self, vals: AvailSet) -> DeductionReason;
    /// Vals must share a cell in this.
    fn fail_must_share(self, vals: AvailSet) -> UnsolveableReason;
    /// The last copy of the given value was eliminated from the row/col/sec.
    fn fail_missing_val(self, val: Val) -> UnsolveableReason;
}

impl RowColSec for Row {
    fn visit(self) -> ReduceStep {
        ReduceStep::RowValsSingularized(self)
    }
    fn deduced(self, vals: AvailSet) -> DeductionReason {
        DeductionReason::UniqueInRow { pos: self, vals }
    }
    fn fail_must_share(self, vals: AvailSet) -> UnsolveableReason {
        UnsolveableReason::RowValsMustShare { pos: self, vals }
    }
    fn fail_missing_val(self, val: Val) -> UnsolveableReason {
        UnsolveableReason::RowMissingVal { pos: self, val }
    }
}

impl RowColSec for Col {
    fn visit(self) -> ReduceStep {
        ReduceStep::ColValsSingularized(self)
    }
    fn deduced(self, vals: AvailSet) -> DeductionReason {
        DeductionReason::UniqueInCol { pos: self, vals }
    }
    fn fail_must_share(self, vals: AvailSet) -> UnsolveableReason {
        UnsolveableReason::ColValsMustShare { pos: self, vals }
    }
    fn fail_missing_val(self, val: Val) -> UnsolveableReason {
        UnsolveableReason::ColMissingVal { pos: self, val }
    }
}

impl RowColSec for Sector {
    fn visit(self) -> ReduceStep {
        ReduceStep::SecValsSingularized(self)
    }
    fn deduced(self, vals: AvailSet) -> DeductionReason {
        DeductionReason::UniqueInSector { pos: self, vals }
    }
    fn fail_must_share(self, vals: AvailSet) -> UnsolveableReason {
        UnsolveableReason::SecValsMustShare { pos: self, vals }
    }
    fn fail_missing_val(self, val: Val) -> UnsolveableReason {
        UnsolveableReason::SecMissingVal { pos: self, val }
    }
}

/// Helper trait for generalizing row-sector and col-sector.
trait SecRowSecCol: Zone + fmt::Debug + Copy + ExtractRem<Avail = AvailCounter> {
    /// Build a reduce step to visit this when the number of remaining values
    /// matches the SIZE.
    fn visit_size_match(self) -> ReduceStep;

    /// Type of the linear direction.
    type Line: ExtractRem<Avail = AvailCounter>;
    /// Gets the line this is a part of.
    fn line(self) -> Self::Line;
    /// Get an iterator over neighbors in the same row/col as self.
    fn line_neighbors(self) -> array::IntoIter<Self, 2>
    where
        Self: Sized;

    /// Gets the sector this is a part of.
    fn sector(self) -> Sector;
    /// Get an iterator over neighbors in the same sector as self.
    fn sec_neighbors(self) -> array::IntoIter<Self, 2>
    where
        Self: Sized;

    /// Visit this sector-row/sector-col as the only one in the row/col
    /// containing some values.
    fn visit_only_in_line(self) -> ReduceStep;
    /// Visit this sector-row/sector-col as the only one in the sector containing
    /// some values.
    fn visit_only_in_sec(self) -> ReduceStep;

    /// Eliminated the given values based on this sector-row/sector-col having
    /// only 3 values left.
    fn deduced_size_match(self, vals: AvailSet) -> DeductionReason;
    /// Eliminated the given values based on this sector-row/sector-col being the
    /// only one in the row/col that could hold the given values.
    fn deduced_only_in_line(self, vals: AvailSet) -> DeductionReason;
    /// Eliminated the given values based on this sector-row/sector-col being the
    /// only one in the sector that could hold the given values.
    fn deduced_only_in_sec(self, vals: AvailSet) -> DeductionReason;
    /// There are too few values left in the sector-row/sector-col to fill it.
    fn fail_too_few_vals(self) -> UnsolveableReason;
}

impl SecRowSecCol for SectorRow {
    fn visit_size_match(self) -> ReduceStep {
        ReduceStep::SecRowTripleized(self)
    }
    type Line = Row;
    fn line(self) -> Self::Line {
        self.row()
    }
    fn line_neighbors(self) -> array::IntoIter<Self, 2> {
        self.row_neighbors()
    }
    fn sector(self) -> Sector {
        SectorRow::sector(&self)
    }
    fn sec_neighbors(self) -> array::IntoIter<Self, 2> {
        self.sector_neighbors()
    }
    fn visit_only_in_line(self) -> ReduceStep {
        ReduceStep::RowOnlySec(self)
    }
    fn visit_only_in_sec(self) -> ReduceStep {
        ReduceStep::SecOnlyRow(self)
    }
    fn deduced_size_match(self, vals: AvailSet) -> DeductionReason {
        DeductionReason::SecRowTriple { pos: self, vals }
    }
    fn deduced_only_in_line(self, vals: AvailSet) -> DeductionReason {
        DeductionReason::RowOnlySec { pos: self, vals }
    }
    fn deduced_only_in_sec(self, vals: AvailSet) -> DeductionReason {
        DeductionReason::SecOnlyRow { pos: self, vals }
    }
    fn fail_too_few_vals(self) -> UnsolveableReason {
        UnsolveableReason::SecRowTooFewVals { pos: self }
    }
}

impl SecRowSecCol for SectorCol {
    fn visit_size_match(self) -> ReduceStep {
        ReduceStep::SecColTripleized(self)
    }
    type Line = Col;
    fn line(self) -> Self::Line {
        self.col()
    }
    fn line_neighbors(self) -> array::IntoIter<Self, 2> {
        self.col_neighbors()
    }
    fn sector(self) -> Sector {
        SectorCol::sector(&self)
    }
    fn sec_neighbors(self) -> array::IntoIter<Self, 2> {
        self.sector_neighbors()
    }
    fn visit_only_in_line(self) -> ReduceStep {
        ReduceStep::ColOnlySec(self)
    }
    fn visit_only_in_sec(self) -> ReduceStep {
        ReduceStep::SecOnlyCol(self)
    }
    fn deduced_size_match(self, vals: AvailSet) -> DeductionReason {
        DeductionReason::SecColTriple { pos: self, vals }
    }
    fn deduced_only_in_line(self, vals: AvailSet) -> DeductionReason {
        DeductionReason::ColOnlySec { pos: self, vals }
    }
    fn deduced_only_in_sec(self, vals: AvailSet) -> DeductionReason {
        DeductionReason::SecOnlyCol { pos: self, vals }
    }
    fn fail_too_few_vals(self) -> UnsolveableReason {
        UnsolveableReason::SecColTooFewVals { pos: self }
    }
}

/// Steps to apply to reduce the remaining values.
/// Reduce steps compare equal if they have the enum Variant and Zone, regardless of
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum ReduceStep {
    /// The given coordinate changed to only have one value left.
    /// Will only be enqueued once for each cell.
    CoordSingularized(Coord),
    /// The given row changed so there is only one slot left that could hold some
    /// value.
    /// May be enqueued and processed more than once for each row.
    RowValsSingularized(Row),
    /// The given col changed so there is only one slot left that could hold some
    /// value.
    /// May be enqueued and processed more than once for each col.
    ColValsSingularized(Col),
    /// The given sector changed so there is only one slot left that could hold
    /// some value.
    /// May be enqueued and processed more than once for each sector.
    SecValsSingularized(Sector),
    /// The given sector-row changed so the number of values left is exactly 3.
    /// Will only be enqueued once per sector-row.
    SecRowTripleized(SectorRow),
    /// The given sector-col changed so the number of values left is exactly 3.
    /// Will only be enqueued once per sector-col.
    SecColTripleized(SectorCol),
    /// The sector row changed so it is the only place left in its row that can
    /// hold one or more values, so those values can be eliminated from the rest
    /// of the sector.
    /// May be enqueued more than once per sector-row.
    RowOnlySec(SectorRow),
    /// The sector row changed so it is the only place left in its sector that
    /// can hold one or more values, so those values can be eliminated from the
    /// rest of the row.
    /// May be enqueued more than once per sector-row.
    SecOnlyRow(SectorRow),
    /// The sector col changed so it is the only place left in its col that can
    /// hold one or more values, so those values can be eliminated from the rest
    /// of the sector.
    /// May be enqueued more than once per sector-col.
    ColOnlySec(SectorCol),
    /// The sector col changed so it is the only place left in its sector that
    /// can hold one or more values, so those values can be eliminated from the
    /// rest of the col.
    /// May be enqueued more than once per sector-col.
    SecOnlyCol(SectorCol),
}

/// Reduce queue which auto-combines certain reduce operations.
struct ReduceQueue {
    /// Min heap of ReduceSteps to be executed.
    pending: BinaryHeap<Reverse<ReduceStep>>,
    /// Hash set used to dedup the heap.
    dedup: HashSet<ReduceStep>,
}

impl ReduceQueue {
    fn new() -> Self {
        Self {
            pending: BinaryHeap::new(),
            dedup: HashSet::new(),
        }
    }

    /// Add a reduce step to the queue if not already there.
    fn push(&mut self, step: ReduceStep) {
        if self.dedup.insert(step) {
            self.pending.push(Reverse(step));
        }
    }

    /// Remove a reduce step from the queue.
    fn pop(&mut self) -> Option<ReduceStep> {
        match self.pending.pop() {
            Some(Reverse(step)) => {
                assert!(self.dedup.remove(&step));
                Some(step)
            }
            None => None,
        }
    }
}

/// Find all reduction rules we should start with for the given board.
fn build_queue(remaining: &RemainingTracker) -> ReduceQueue {
    let mut queue = ReduceQueue::new();
    for (coord, avail) in remaining.get::<Coord>().iter() {
        if avail.is_single() {
            queue.push(ReduceStep::CoordSingularized(coord))
        }
    }
    build_row_col_sec_queue::<Row>(remaining, &mut queue);
    build_row_col_sec_queue::<Col>(remaining, &mut queue);
    build_row_col_sec_queue::<Sector>(remaining, &mut queue);
    build_secrow_seccol_queue::<SectorRow>(remaining, &mut queue);
    build_secrow_seccol_queue::<SectorCol>(remaining, &mut queue);
    queue
}

/// Adds entries to vist any row/col/sector that already has entries which can
/// only occupy a single cell.
fn build_row_col_sec_queue<Z: RowColSec>(rem: &RemainingTracker, queue: &mut ReduceQueue) {
    for (rcs, avail) in rem.get::<Z>().iter() {
        if avail.counts().any(|(_, &count)| count == 1) {
            queue.push(rcs.visit());
        }
    }
}

/// Adds entries to visit any sector-row/sector-col that already reductions available.
fn build_secrow_seccol_queue<Z: SecRowSecCol>(rem: &RemainingTracker, queue: &mut ReduceQueue) {
    for (srsc, avail) in rem.get::<Z>().iter() {
        if avail.avail().len() == Z::SIZE {
            queue.push(srsc.visit_size_match());
        }
        if avail
            .counts()
            .any(|(val, &count)| count == rem[srsc.line()][val])
        {
            queue.push(srsc.visit_only_in_line());
        }
        if avail
            .counts()
            .any(|(val, &count)| count == rem[srsc.sector()][val])
        {
            queue.push(srsc.visit_only_in_sec());
        }
    }
}
