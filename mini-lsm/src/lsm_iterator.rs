use anyhow::Result;

use crate::{
    iterators::{StorageIterator, merge_iterator::MergeIterator},
    mem_table::MemTableIterator,
};

/// Represents the internal type for an LSM iterator.
type LsmIteratorInner = MergeIterator<MemTableIterator>;

pub struct LsmIterator {
    inner: LsmIteratorInner,
}

impl LsmIterator {
    pub(crate) fn new(iter: LsmIteratorInner) -> Result<Self> {
        Ok(Self { inner: iter })
    }
}

impl StorageIterator for LsmIterator {
    type KeyType<'a> = &'a [u8];

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

/// A wrapper around existing iterator, will prevent users from calling `next` when the iterator is
/// invalid. If an iterator is already invalid, `next` does not do anything. If `next` returns an
/// error, `is_valid` should return false, and `next` should always return an error.
pub struct FusedIterator<I>
where
    I: StorageIterator,
{
    iter: I,
    has_errored: bool,
}

impl<I> FusedIterator<I>
where
    I: StorageIterator,
{
    pub fn new(iter: I) -> Self {
        Self {
            iter,
            has_errored: false,
        }
    }
}

impl<I> StorageIterator for FusedIterator<I>
where
    I: StorageIterator,
{
    type KeyType<'a>
        = I::KeyType<'a>
    where
        Self: 'a;

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
