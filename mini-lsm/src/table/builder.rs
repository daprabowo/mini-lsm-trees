use std::{path::Path, sync::Arc};

use anyhow::Result;

use crate::{
    block::{BlockCache, builder::BlockBuilder},
    key::KeySlice,
    table::{BlockMeta, SsTable},
};

/// Builds an SSTable from key-value pairs.
pub struct SsTableBuilder {
    builder: BlockBuilder,
    first_key: Vec<u8>,
    last_key: Vec<u8>,
    data: Vec<u8>,
    pub(crate) meta: Vec<BlockMeta>,
    block_size: usize,
}

impl SsTableBuilder {
    /// Create an SSTable builder based on target block size.
    pub fn new(block_size: usize) -> Self {
        unimplemented!()
    }

    /// Adds a key-value pair to SSTable.
    pub fn add(&mut self, key: KeySlice, value: &[u8]) {
        // NOTE: You should split a new block when the current block is full. (`std::mem::replace`
        // may be helpful here).
        unimplemented!()
    }

    /// Builds the SSTable and writes it to the given path.
    /// Use the `FileObject` structure to manipulate the disk objects.
    pub fn build<P>(
        #[allow(unused_mut)] mut self,
        id: usize,
        block_cache: Option<Arc<BlockCache>>,
        path: P,
    ) -> Result<SsTable>
    where
        P: AsRef<Path>,
    {
        unimplemented!()
    }

    #[cfg(test)]
    pub(crate) fn build_for_test<P>(self, path: P) -> Result<SsTable>
    where
        P: AsRef<Path>,
    {
        self.build(0, None, path)
    }
}
