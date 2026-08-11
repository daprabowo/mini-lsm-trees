use std::ops::{Bound, RangeBounds};

pub trait LsmRange {
    fn start(&self) -> Bound<&[u8]>;
    fn end(&self) -> Bound<&[u8]>;
}

impl LsmRange for std::ops::RangeFull {
    fn start(&self) -> Bound<&[u8]> {
        Bound::Unbounded
    }

    fn end(&self) -> Bound<&[u8]> {
        Bound::Unbounded
    }
}

impl LsmRange for &[u8] {
    fn start(&self) -> Bound<&[u8]> {
        Bound::Included(self)
    }

    fn end(&self) -> Bound<&[u8]> {
        Bound::Included(self)
    }
}

macro_rules! impl_lsm_range {
    ($range:ty) => {
        impl<K: AsRef<[u8]>> LsmRange for $range {
            fn start(&self) -> Bound<&[u8]> {
                match self.start_bound() {
                    Bound::Included(k) => Bound::Included(k.as_ref()),
                    Bound::Excluded(k) => Bound::Excluded(k.as_ref()),
                    Bound::Unbounded => Bound::Unbounded,
                }
            }

            fn end(&self) -> Bound<&[u8]> {
                match self.end_bound() {
                    Bound::Included(k) => Bound::Included(k.as_ref()),
                    Bound::Excluded(k) => Bound::Excluded(k.as_ref()),
                    Bound::Unbounded => Bound::Unbounded,
                }
            }
        }
    };
}

impl_lsm_range!(std::ops::Range<K>);
impl_lsm_range!(std::ops::RangeInclusive<K>);
impl_lsm_range!(std::ops::RangeFrom<K>);
impl_lsm_range!(std::ops::RangeTo<K>);
impl_lsm_range!(std::ops::RangeToInclusive<K>);
