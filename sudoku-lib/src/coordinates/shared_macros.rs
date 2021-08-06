macro_rules! rowcol_fromint {
    ($imp:ty, $max:expr, $name:literal, $($t:ty),*) => {
        $(
            impl From<$t> for $imp {
                fn from(val: $t) -> Self {
                    assert!(
                        (0 as $t .. $max as $t).contains(&val),
                        concat!($name, " must be in range [0, {}), got {}"),
                        $max, val,
                    );
                    Self(val as u8)
                }
            }
        )*
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
