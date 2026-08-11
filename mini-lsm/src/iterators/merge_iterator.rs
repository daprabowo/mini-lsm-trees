use std::{
    cmp,
    collections::{BinaryHeap, binary_heap::PeekMut},
};

use anyhow::Result;

use crate::{iterators::StorageIterator, key::KeySlice};

struct HeapWrapper<I: StorageIterator>(pub usize, pub Box<I>);

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
        let mut iters: BinaryHeap<HeapWrapper<I>> = iters
            .into_iter()
            .enumerate()
            .filter(|(_, iter)| iter.is_valid())
            .map(|(idx, iter)| HeapWrapper(idx, iter))
            .collect();
        let current = iters.pop();
        Self { iters, current }
    }
}

impl<I> StorageIterator for MergeIterator<I>
where
    I: 'static + for<'a> StorageIterator<KeyType<'a> = KeySlice<'a>>,
{
    type KeyType<'a> = KeySlice<'a>;

    fn next(&mut self) -> Result<()> {
        let current = match self.current.as_mut() {
            Some(c) => c,
            None => return Ok(()),
        };

        while let Some(mut top) = self.iters.peek_mut() {
            if top.1.key() == current.1.key() {
                // top.1.next()?;
                //
                // if !top.1.is_valid() {
                //     PeekMut::pop(top);
                // }

                if let Err(e) = top.1.next() {
                    PeekMut::pop(top);
                    return Err(e);
                }

                if !top.1.is_valid() {
                    PeekMut::pop(top);
                } else {
                    std::mem::drop(top);
                }
            } else {
                break;
            }
        }

        current.1.next()?;

        if !current.1.is_valid() {
            self.current = self.iters.pop();
        } else {
            if let Some(mut top) = self.iters.peek_mut()
                && *current < *top
            {
                std::mem::swap(current, &mut *top);
            }
        }

        Ok(())
    }

    fn key(&self) -> Self::KeyType<'_> {
        self.current.as_ref().unwrap().1.key()
    }

    fn value(&self) -> &[u8] {
        self.current.as_ref().unwrap().1.value()
    }

    fn is_valid(&self) -> bool {
        self.current.is_some()
    }
}
