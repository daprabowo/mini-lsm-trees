use anyhow::Result;

use crate::iterators::StorageIterator;

pub struct TwoMergeIterator<L, R>
where
    L: StorageIterator,
    R: StorageIterator,
{
    lhs: L,
    rhs: R,
}

impl<L, R> TwoMergeIterator<L, R>
where
    L: 'static + StorageIterator,
    R: 'static + for<'a> StorageIterator<KeyType<'a> = L::KeyType<'a>>,
{
    pub fn create(lhs: L, rhs: R) -> Result<Self> {
        unimplemented!()
    }
}

impl<L, R> StorageIterator for TwoMergeIterator<L, R>
where
    L: 'static + StorageIterator,
    R: 'static + for<'a> StorageIterator<KeyType<'a> = L::KeyType<'a>>,
{
    type KeyType<'a> = R::KeyType<'a>;

    fn next(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn key(&self) -> Self::KeyType<'_> {
        unimplemented!()
    }

    fn value(&self) -> &[u8] {
        unimplemented!()
    }

    fn is_valid(&self) -> bool {
        unimplemented!()
    }
}
