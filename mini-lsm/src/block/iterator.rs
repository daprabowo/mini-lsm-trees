use std::sync::Arc;

use crate::{
    block::Block,
    key::{KeySlice, KeyVec},
};

/// Iterates on a block.
pub struct BlockIterator {
    /// The internal `Block`, wrapped by an `Arc`.
    block: Arc<Block>,
    /// The current key, empty represent the iterator is invalid.
    key: KeyVec,
    /// The current value range in the block.data, corresponds to the current key.
    value_range: (usize, usize),
    /// Current index of the key-value pair, should be in range of [0, num_of_elements].
    idx: usize,
    /// The first key in the block
    first_key: KeyVec,
}

impl BlockIterator {
    fn new(block: Arc<Block>) -> Self {
        Self {
            block,
            key: KeyVec::new(),
            value_range: (0, 0),
            idx: 0,
            first_key: KeyVec::new(),
        }
    }

    /// Creates a block iterator and seek to the first entry.
    pub fn create_and_seek_to_first(block: Arc<Block>) -> Self {
        unimplemented!()
    }

    /// Creates a block iterator and seek to the first key that >= `key`.
    pub fn create_and_seek_to_key(block: Arc<Block>, key: KeySlice) -> Self {
        unimplemented!()
    }

    /// Returns the key of the current entry.
    pub fn key(&self) -> KeySlice<'_> {
        unimplemented!()
    }

    /// Returns the value of the current entry.
    pub fn value(&self) -> &[u8] {
        unimplemented!()
    }

    /// Returns true if the iterator is valid.
    pub fn is_valid(&self) -> bool {
        // NOTE: You may want to make use of `key`
        unimplemented!()
    }

    /// Seeks to the first key in the block.
    pub fn seek_to_first(&mut self) {
        unimplemented!()
    }

    /// Move to the next key in the block.
    pub fn next(&mut self) {
        unimplemented!()
    }

    /// Seek to the first key that >= `key`.
    pub fn seek_to_key(&mut self, key: KeySlice) {
        // NOTE: You should assume the key-value pairs in the block are sorted when being added by
        // callers
        unimplemented!()
    }
}
