use std::sync::Arc;

use bytes::{BufMut, Bytes};

use crate::{
    block::Block,
    key::{KeySlice, KeyVec},
};

/// Builds a block.
pub struct BlockBuilder {
    /// Offsets of each key-value entries.
    offsets: Vec<u16>,
    /// All serialized key-value pairs in the block.
    data: Vec<u8>,
    /// The expected block size.
    block_size: usize,
    /// The first key in the block.
    first_key: KeyVec,
}

impl BlockBuilder {
    /// Creates a new block builder.
    pub fn new(block_size: usize) -> Self {
        Self {
            offsets: Vec::new(),
            data: Vec::new(),
            block_size,
            first_key: KeyVec::new(),
        }
    }

    /// Adds a key-value pair to the block. Returns false when the block is full.
    /// You may find the `bytes::BufMut` trait useful for manipulating binary data.
    #[must_use = "empty block builder is not allowed"]
    pub fn add(&mut self, key: KeySlice, value: &[u8]) -> bool {
        let entry_size = std::mem::size_of::<u16>()
            + key.len()
            + std::mem::size_of::<u16>()
            + value.len()
            + std::mem::size_of::<u16>();

        let estimated_size = self.data.len()
            + (self.offsets.len() * std::mem::size_of::<u16>())
            + entry_size
            + std::mem::size_of::<u16>();

        if estimated_size >= self.block_size && !self.is_empty() {
            return false;
        }

        if self.is_empty() {
            self.first_key = key.to_key_vec();
        }

        self.offsets.push(self.data.len() as u16);

        self.data.put_u16(key.len() as u16);
        self.data.put_slice(key.raw_ref());
        self.data.put_u16(value.len() as u16);
        self.data.put_slice(value);

        true
    }

    /// Check if there is not key-value pair in the block.
    pub fn is_empty(&self) -> bool {
        self.offsets.is_empty()
    }

    /// Finalize the block.
    pub fn build(self) -> Block {
        assert!(!self.is_empty(), "unable to build an empty block");
        Block {
            data: Bytes::from(self.data),
            offsets: Arc::from(self.offsets),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_build_single_key() {
        let mut builder = Block::builder(16);
        assert!(builder.add(KeySlice::for_testing_from_slice_no_ts(b"123"), b"123456"));
        builder.build();
    }

    #[test]
    fn test_block_build_full() {
        let mut builder = Block::builder(16);
        assert!(builder.add(KeySlice::for_testing_from_slice_no_ts(b"11"), b"11"));
        assert!(!builder.add(KeySlice::for_testing_from_slice_no_ts(b"22"), b"22"));
        builder.build();
    }

    #[test]
    fn test_block_build_large_1() {
        let mut builder = Block::builder(16);
        assert!(builder.add(
            KeySlice::for_testing_from_slice_no_ts(b"11"),
            &b"1".repeat(100)
        ));
        builder.build();
    }

    #[test]
    fn test_block_build_large_2() {
        let mut builder = Block::builder(16);
        assert!(builder.add(KeySlice::for_testing_from_slice_no_ts(b"11"), b"1"));
        assert!(!builder.add(
            KeySlice::for_testing_from_slice_no_ts(b"11"),
            &b"1".repeat(100)
        ));
        builder.build();
    }
}
