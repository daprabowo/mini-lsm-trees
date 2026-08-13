pub mod builder;
pub mod iterator;

use std::{sync::Arc, u16};

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::block::builder::BlockBuilder;

pub type BlockCache = moka::sync::Cache<(usize, usize), Arc<Block>>;

/// A block is the smallest unit of read and caching in LSM tree. It is a collection of sorted
/// key-value pairs.
pub struct Block {
    pub(crate) data: Vec<u8>,
    pub(crate) offsets: Vec<u16>,
}

impl Block {
    pub fn builder(block_size: usize) -> BlockBuilder {
        BlockBuilder::new(block_size)
    }

    /// Encode the internal data to the data layout illustrated in the course
    pub fn encode(&self) -> Bytes {
        let estimated_size = self.data.len()
            + (self.offsets.len() * std::mem::size_of::<u16>())
            + std::mem::size_of::<u16>();

        let mut buf = BytesMut::with_capacity(estimated_size);
        buf.put(&self.data[..]);

        for &offset in &self.offsets {
            buf.put_u16(offset);
        }

        buf.put_u16(self.offsets.len() as u16);

        buf.freeze()
    }

    /// Decode from the data layout, transform the inut `data` to a single `Block`
    pub fn decode(data: &[u8]) -> Self {
        let entry_count_start = data.len() - std::mem::size_of::<u16>();
        let entry_count = (&data[entry_count_start..data.len()]).get_u16() as usize;

        let offsets_start = entry_count_start - (entry_count * std::mem::size_of::<u16>());
        let mut offsets = Vec::with_capacity(entry_count);
        let mut offsets_ptr = &data[offsets_start..entry_count_start];

        for _ in 0..entry_count {
            offsets.push(offsets_ptr.get_u16());
        }

        Self {
            data: data[..offsets_start].to_vec(),
            offsets,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::key::KeyVec;

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

    #[test]
    fn test_block_encode() {
        let block = generate_block();
        block.encode();
    }

    #[test]
    fn test_block_decode() {
        let block = generate_block();
        let encoded = block.encode();
        let decoded = Block::decode(&encoded);
        assert_eq!(block.offsets, decoded.offsets);
        assert_eq!(block.data, decoded.data);
    }
}
