//! Different types of coordinates on the board -- individual cells, sectors,
//! rows, and columns.
pub use column::Col;
pub use intersections::Intersect;
pub use intersections::rowsec::SectorRow;
pub use intersections::colsec::SectorCol;
pub use coord::Coord;
pub use row::Row;
pub use sector::Sector;
pub use zone::Zone;
pub(crate) use zone::{FixedSizeIndexable, ZoneContaining};

#[macro_use]
mod shared_macros;

mod column;
mod coord;
mod intersections;
mod row;
mod sector;
mod zone;
