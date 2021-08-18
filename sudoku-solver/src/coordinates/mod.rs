//! Different types of coordinates on the board -- individual cells, sectors,
//! rows, and columns.
use std::fmt;

use thiserror::Error;

pub use column::Col;
pub use coord::Coord;
pub use intersections::colsec::SectorCol;
pub use intersections::rowsec::SectorRow;
pub use intersections::Intersect;
pub use row::Row;
pub use sector::Sector;
pub use zone::Zone;
pub(crate) use zone::{Coords, FixedSizeIndexable, ZoneContaining};

#[macro_use]
mod shared_macros;

mod column;
mod coord;
mod intersections;
mod row;
mod sector;
mod zone;

/// Error used when creating a coordinate type from a number that's out of range.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Error)]
#[error("value {0:?} is out of range")]
pub struct OutOfRange<T: fmt::Debug>(pub T);