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

#[cfg(feature = "serde")]
mod serde_utils {
    use std::convert::TryFrom;
    use std::fmt;

    use serde::{de, Deserializer};

    use crate::{Col, Row};

    macro_rules! deserialize_base_rowcol {
        ($name:ident, $t:ident) => {
            pub(super) fn $name<'de, D>(deserializer: D) -> Result<$t, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct Visitor;

                impl<'de> de::Visitor<'de> for Visitor {
                    type Value = $t;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("0, 3, or 6")
                    }

                    fn visit_i64<E: de::Error>(self, v: i64) -> Result<Self::Value, E> {
                        $t::try_from(v)
                            .map_err(|_| E::invalid_value(de::Unexpected::Signed(v), &self))
                    }

                    fn visit_i128<E: de::Error>(self, v: i128) -> Result<Self::Value, E> {
                        $t::try_from(v)
                            .map_err(|_| E::invalid_value(de::Unexpected::Other("i128"), &self))
                    }

                    fn visit_u64<E: de::Error>(self, v: u64) -> Result<Self::Value, E> {
                        $t::try_from(v)
                            .map_err(|_| E::invalid_value(de::Unexpected::Unsigned(v), &self))
                    }

                    fn visit_u128<E: de::Error>(self, v: u128) -> Result<Self::Value, E> {
                        $t::try_from(v)
                            .map_err(|_| E::invalid_value(de::Unexpected::Other("u128"), &self))
                    }
                }

                deserializer.deserialize_u8(Visitor)
            }
        };
    }

    deserialize_base_rowcol!(deserialize_base_row, Row);
    deserialize_base_rowcol!(deserialize_base_col, Col);
}
