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

macro_rules! zone_coords_iter {
    ($it:ty) => {
        impl Iterator for $it {
            type Item = Coord;

            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.range.next().map(|val| self.build_coord(val))
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.range.size_hint()
            }

            #[inline]
            fn nth(&mut self, n: usize) -> Option<Self::Item> {
                self.range.nth(n).map(|val| self.build_coord(val))
            }

            #[inline]
            fn last(mut self) -> Option<Self::Item> {
                self.next_back()
            }
        }

        impl ExactSizeIterator for $it {}

        impl DoubleEndedIterator for $it {
            fn next_back(&mut self) -> Option<Self::Item> {
                self.range.next_back().map(|val| self.build_coord(val))
            }

            fn nth_back(&mut self, n: usize) -> Option<Self::Item> {
                self.range.nth_back(n).map(|val| self.build_coord(val))
            }
        }

        impl std::iter::FusedIterator for $it {}
    };
}