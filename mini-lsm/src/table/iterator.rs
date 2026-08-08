use std::sync::Arc;

use anyhow::Result;

use crate::{
    block::iterator::BlockIterator, iterators::StorageIterator, key::KeySlice, table::SsTable,
};

pub struct SsTableIterator {
    table: Arc<SsTable>,
    block_iter: BlockIterator,
    block_idx: usize,
}

impl SsTableIterator {
    /// Create a new iterator and seek to the first key-value pair in the first data block.
    pub fn create_and_seek_to_first(table: Arc<SsTable>) -> Result<Self> {
        unimplemented!()
    }

    /// Seek to the first key-value pair in the first data block.
    pub fn seek_to_first(&mut self) -> Result<()> {
        unimplemented!()
    }

    /// Create a new iterator and seek to the first key-value pair which >= `key`.
    pub fn create_and_seek_to_key(table: Arc<SsTable>, key: KeySlice) -> Result<Self> {
        unimplemented!()
    }

    /// Seek to the first key-value pair which >= `key`.
    pub fn seek_to_key(&mut self, key: KeySlice) -> Result<Self> {
        // NOTE: You probably want to review the handout for detailed explanation when implementing
        // this function
        unimplemented!()
    }
}

impl StorageIterator for SsTableIterator {
    type KeyType<'a> = KeySlice<'a>;

    /// Move to the next `key` in the block.
    fn next(&mut self) -> Result<()> {
        // NOTE: You may want to check if the current block iterator is valid after the move.
        unimplemented!()
    }

    /// Return the `key` that'a held by the underlying block iterator.
    fn key(&self) -> Self::KeyType<'_> {
        unimplemented!()
    }

    /// Return the `value` that's held by the underlying block iterator.
    fn value(&self) -> &[u8] {
        unimplemented!()
    }

    /// Return whether the current block iterator is valid or not.
    fn is_valid(&self) -> bool {
        unimplemented!()
    }
}
