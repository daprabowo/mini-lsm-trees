pub mod builder;
pub mod iterator;

use std::sync::Arc;

use bytes::Bytes;

pub type BlockCache = moka::sync::Cache<(usize, usize), Arc<Block>>;

/// A block is the smallest unit of read and caching in LSM tree. It is a collection of sorted
/// key-value pairs.
pub struct Block {
    pub(crate) data: Vec<u8>,
    pub(crate) offsets: Vec<u8>,
}

impl Block {
    /// Encode the internal data to the data layout illustrated in the course
    pub fn encode(&self) -> Bytes {
        // NOTE: You may want to recheck if any of the expected field is missing from your output
        unimplemented!()
    }

    /// Decode from the data layout, transform the inut `data` to a single `Block`
    pub fn decode(data: &[u8]) -> Self {
        unimplemented!()
    }
}
