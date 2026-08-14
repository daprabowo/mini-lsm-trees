use std::sync::Arc;

use bytes::Buf;

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
        let mut this = Self::new(block);
        this.seek_to_first();
        this
    }

    /// Creates a block iterator and seek to the first key that >= `key`.
    pub fn create_and_seek_to_key(block: Arc<Block>, key: KeySlice) -> Self {
        let mut this = Self::new(block);
        this.seek_to_key(key);
        this
    }

    /// Move to the next key in the block.
    pub fn next(&mut self) {
        self.seek_to_index(self.idx + 1);
    }

    pub fn prev(&mut self) {
        if self.idx == 0 {
            self.key.clear();
            self.value_range = (0, 0);
            return;
        }

        self.seek_to_index(self.idx - 1);
    }

    /// Returns the key of the current entry.
    pub fn key(&self) -> KeySlice<'_> {
        self.key.as_key_slice()
    }

    /// Returns the value of the current entry.
    pub fn value(&self) -> &[u8] {
        &self.block.data[self.value_range.0..self.value_range.1]
    }

    /// Returns true if the iterator is valid.
    pub fn is_valid(&self) -> bool {
        !self.key.is_empty()
    }

    fn seek_to_index(&mut self, idx: usize) {
        if idx >= self.block.offsets.len() {
            self.key.clear();
            self.value_range = (0, 0);
            self.idx = idx;
            return;
        }

        let offset = self.block.offsets[idx] as usize;
        let mut data_ptr = &self.block.data[offset..];

        if data_ptr.len() < std::mem::size_of::<u16>() {
            self.key.clear();
            self.value_range = (0, 0);
            return;
        }

        let key_len = data_ptr.get_u16() as usize;

        if data_ptr.len() < key_len {
            self.key.clear();
            self.value_range = (0, 0);
            return;
        }

        let key = &data_ptr[..key_len];
        data_ptr.advance(key_len);

        if data_ptr.len() < std::mem::size_of::<u16>() {
            self.key.clear();
            self.value_range = (0, 0);
            return;
        }

        let value_len = data_ptr.get_u16() as usize;

        if data_ptr.len() < value_len {
            self.key.clear();
            self.value_range = (0, 0);
            return;
        }

        let value_start =
            offset + std::mem::size_of::<u16>() + key_len + std::mem::size_of::<u16>();
        let value_end = value_start + value_len;

        self.idx = idx;
        self.key.set_from_slice(KeySlice::from_slice(key));
        self.value_range = (value_start, value_end);
    }

    /// Seeks to the first key in the block.
    pub fn seek_to_first(&mut self) {
        self.seek_to_index(0);
        if self.is_valid() {
            self.first_key.set_from_slice(self.key.as_key_slice());
        }
    }

    /// Seek to the first key that >= `key`.
    pub fn seek_to_key(&mut self, key: KeySlice) {
        let idx = self.block.offsets.partition_point(|&offset| {
            let mut data_ptr = &self.block.data[(offset as usize)..];
            let key_len = data_ptr.get_u16() as usize;
            let current_key = &data_ptr[..key_len];
            KeySlice::from_slice(current_key) < key
        });

        self.seek_to_index(idx);
    }
}

#[cfg(test)]
mod tests {
    use bytes::Bytes;

    use super::*;

    fn key_of(idx: usize) -> KeyVec {
        KeyVec::for_testing_from_vec_no_ts(format!("key_{:03}", idx * 5).into_bytes())
    }

    fn value_of(idx: usize) -> Vec<u8> {
        format!("value_{:010}", idx).into_bytes()
    }

    fn num_of_keys() -> usize {
        100
    }

    fn generate_block() -> Block {
        let mut builder = Block::builder(10000);
        for idx in 0..num_of_keys() {
            let key = key_of(idx);
            let value = value_of(idx);
            assert!(builder.add(key.as_key_slice(), &value[..]));
        }
        builder.build()
    }

    fn as_bytes(x: &[u8]) -> Bytes {
        Bytes::copy_from_slice(x)
    }

    #[test]
    fn test_block_iterator() {
        let block = Arc::new(generate_block());
        let mut iter = BlockIterator::create_and_seek_to_first(block);

        for i in 0..num_of_keys() {
            let key = iter.key();
            let value = iter.value();
            assert!(iter.is_valid());
            assert_eq!(
                key.for_testing_key_ref(),
                key_of(i).for_testing_key_ref(),
                "expected key: {:?}, actual key: {:?}",
                as_bytes(key_of(i).for_testing_key_ref()),
                as_bytes(key.for_testing_key_ref())
            );
            assert_eq!(
                value,
                value_of(i),
                "expected value: {:?}, actual value: {:?}",
                as_bytes(&value_of(i)),
                as_bytes(value)
            );
            iter.next();
        }

        for i in (0..num_of_keys()).rev() {
            iter.prev();
            let key = iter.key();
            let value = iter.value();
            assert!(iter.is_valid());
            assert_eq!(
                key.for_testing_key_ref(),
                key_of(i).for_testing_key_ref(),
                "expected key: {:?}, actual key: {:?}",
                as_bytes(key_of(i).for_testing_key_ref()),
                as_bytes(key.for_testing_key_ref())
            );
            assert_eq!(
                value,
                value_of(i),
                "expected value: {:?}, actual value: {:?}",
                as_bytes(&value_of(i)),
                as_bytes(value)
            );
        }
    }

    #[test]
    fn test_block_seek_key() {
        let block = Arc::new(generate_block());
        let mut iter = BlockIterator::create_and_seek_to_key(block, key_of(0).as_key_slice());
        for offset in 1..=5 {
            for i in 0..num_of_keys() {
                let key = iter.key();
                let value = iter.value();
                assert!(iter.is_valid());
                assert_eq!(
                    key.for_testing_key_ref(),
                    key_of(i).for_testing_key_ref(),
                    "expected key: {:?}, actual key: {:?}",
                    as_bytes(key_of(i).for_testing_key_ref()),
                    as_bytes(key.for_testing_key_ref())
                );
                assert_eq!(
                    value,
                    value_of(i),
                    "expected value: {:?}, actual value: {:?}",
                    as_bytes(&value_of(i)),
                    as_bytes(value)
                );
                iter.seek_to_key(KeySlice::for_testing_from_slice_no_ts(
                    &format!("key_{:03}", i * 5 + offset).into_bytes(),
                ));
            }
            iter.seek_to_key(KeySlice::for_testing_from_slice_no_ts(b"k"));
        }
    }
}
