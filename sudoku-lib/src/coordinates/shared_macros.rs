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

macro_rules! zone_indexes_iter {
    ($it:ty) => {
        impl Iterator for $it {
            type Item = Coord;

            #[inline]
            fn next(&mut self) -> Option<Coord> {
                self.range.next().map(|val| self.build_coord(val))
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.range.size_hint()
            }

            #[inline]
            fn nth(&mut self, n: usize) -> Option<Coord> {
                self.range.nth(n).map(|val| self.build_coord(val))
            }

            #[inline]
            fn last(mut self) -> Option<Coord> {
                self.range.next_back().map(|val| self.build_coord(val))
            }
        }

        impl std::iter::ExactSizeIterator for $it {}

        impl std::iter::DoubleEndedIterator for $it {
            fn next_back(&mut self) -> Option<Coord> {
                self.range.next_back().map(|val| self.build_coord(val))
            }

            fn nth_back(&mut self, n: usize) -> Option<Coord> {
                self.range.nth_back(n).map(|val| self.build_coord(val))
            }
        }
    };
}

macro_rules! zone_all_iter {
    ($it:ty, $zone:ty) => {
        impl Iterator for $it {
            type Item = $zone;

            #[inline]
            fn next(&mut self) -> Option<$zone> {
                self.0.next().map(|val| Self::build_zone(val))
            }

            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.0.size_hint()
            }

            #[inline]
            fn nth(&mut self, n: usize) -> Option<$zone> {
                self.0.nth(n).map(|val| Self::build_zone(val))
            }

            #[inline]
            fn last(mut self) -> Option<$zone> {
                self.0.next_back().map(|val| Self::build_zone(val))
            }
        }

        impl std::iter::ExactSizeIterator for $it {}

        impl std::iter::DoubleEndedIterator for $it {
            fn next_back(&mut self) -> Option<$zone> {
                self.0.next_back().map(|val| Self::build_zone(val))
            }

            fn nth_back(&mut self, n: usize) -> Option<$zone> {
                self.0.nth_back(n).map(|val| Self::build_zone(val))
            }
        }
    };
}