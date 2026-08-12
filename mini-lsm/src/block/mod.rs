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
        let mut offsets_raw = &data[offsets_start..entry_count_start];

        for _ in 0..entry_count {
            offsets.push(offsets_raw.get_u16());
        }

        Self {
            data: data[..offsets_start].to_vec(),
            offsets,
        }
    }
}
