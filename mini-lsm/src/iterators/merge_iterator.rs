use std::{cmp, collections::BinaryHeap};

use anyhow::Result;

use crate::{iterators::StorageIterator, key::KeySlice};

struct HeapWrapper<I>(pub usize, pub Box<I>)
where
    I: StorageIterator;

impl<I> PartialEq for HeapWrapper<I>
where
    I: StorageIterator,
{
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == cmp::Ordering::Equal
    }
}

impl<I> Eq for HeapWrapper<I> where I: StorageIterator {}

impl<I> PartialOrd for HeapWrapper<I>
where
    I: StorageIterator,
{
    fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<I> Ord for HeapWrapper<I>
where
    I: StorageIterator,
{
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.1
            .key()
            .cmp(&other.1.key())
            .then(self.0.cmp(&other.0))
            .reverse()
    }
}

/// Merge multiple iterators of the same type. If the same key occurs multiple times in some
/// iterators, prefer the one with smaller index.
pub struct MergeIterator<I>
where
    I: StorageIterator,
{
    iters: BinaryHeap<HeapWrapper<I>>,
    current: Option<HeapWrapper<I>>,
}

impl<I> MergeIterator<I>
where
    I: StorageIterator,
{
    pub fn create(iters: Vec<Box<I>>) -> Self {
        unimplemented!()
    }
}

impl<I> StorageIterator for MergeIterator<I>
where
    I: 'static + for<'a> StorageIterator<KeyType<'a> = KeySlice<'a>>,
{
    type KeyType<'a> = KeySlice<'a>;

    fn key(&self) -> Self::KeyType<'_> {
        unimplemented!()
    }

    fn value(&self) -> &[u8] {
        unimplemented!()
    }

    fn is_valid(&self) -> bool {
        unimplemented!()
    }

    fn next(&self) -> Result<()> {
        unimplemented!()
    }
}
