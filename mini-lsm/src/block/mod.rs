pub mod builder;
pub mod iterator;

use bytes::Bytes;

/// A block is the smallest unit of read and caching in LSM tree. It is a collection of sorted
/// key-value pairs.
pub struct Block {
    pub(crate) data: Vec<u8>,
    pub(crate) offsets: Vec<u8>,
}

impl Block {
    /// Encode the internal data to the data layout illustrated in the course
    /// Note: You may want to recheck if any of the expected field is missing from your output
    pub fn encode(&self) -> Bytes {
        unimplemented!()
    }

    /// Decode from the data layout, transform the inut `data` to a single `Block`
    pub fn decode(&self) -> Self {
        unimplemented!()
    }
}
