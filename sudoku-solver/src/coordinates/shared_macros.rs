macro_rules! rowcol_fromint {
    ($imp:ty, $max:expr, $name:literal, $($t:ty),*) => {
        $(
            impl std::convert::TryFrom<$t> for $imp {
                type Error = crate::OutOfRange<$t>;

                fn try_from(val: $t) -> Result<Self, Self::Error> {
                    if (0 as $t .. $max as $t).contains(&val) {
                        Ok(Self(val as u8))
                    } else {
                        Err(crate::OutOfRange(val))
                    }
                }
            }

            impl From<$imp> for $t {
                fn from(val: $imp) -> $t {
                    val.0 as $t
                }
            }
        )*
    };
}

macro_rules! fixed_size_indexable_into_iter {
    ($t:ty) => {
        impl IntoIterator for $t {
            type Item = Coord;
            type IntoIter = crate::coordinates::Coords<$t>;

            fn into_iter(self) -> Self::IntoIter {
                self.into()
            }
        }
    };
}

macro_rules! reciprocal_intersect {
    (<$z1:ty> for $z2:ty) => {
        impl Intersect<$z1> for $z2 {
            type Intersection = <$z1 as Intersect<$z2>>::Intersection;

            fn intersect(self, other: $z1) -> Option<Self::Intersection> {
                other.intersect(self)
            }
        }
    };
}

#[cfg(test)]
macro_rules! assert_sorted {
    ($v:ident) => {
        let mut sorted = $v.clone();
        sorted.sort();
        assert_eq!($v, sorted);
    };
}
