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
        )*
    };
}

macro_rules! rowcol_named_consts {
    ($imp:ident) => {
        impl $imp {
            pub const ZERO: $imp = $imp(0);
            pub const ONE: $imp = $imp(1);
            pub const TWO: $imp = $imp(2);
            pub const THREE: $imp = $imp(3);
            pub const FOUR: $imp = $imp(4);
            pub const FIVE: $imp = $imp(5);
            pub const SIX: $imp = $imp(6);
            pub const SEVEN: $imp = $imp(7);
            pub const EIGHT: $imp = $imp(8);
        }
    }
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
