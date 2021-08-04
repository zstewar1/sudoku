//! Different types of coordinates on the board -- individual cells, sectors,
//! rows, and columns.
pub use column::Col;
pub use coord::Coord;
pub use row::Row;
pub use sector::Sector;
pub use zone::Zone;

#[macro_use]
mod shared_macros;

mod column;
mod coord;
mod row;
mod sector;
mod zone;
