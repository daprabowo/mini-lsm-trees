use std::sync::Arc;

use anyhow::Result;

use crate::{
    iterators::StorageIterator,
    key::KeySlice,
    table::{SsTable, SsTableIterator},
};

/// Concat multiple iterators ordered in key order and their key ranges do not overlap. We do not
/// want to create the iterators when initializing this iterator to reduce the overhead of seeking.
pub struct SstConcatIterator {
    current: Option<SsTableIterator>,
    next_sst_idx: usize,
    sstables: Vec<Arc<SsTable>>,
}

impl SstConcatIterator {
    pub fn create_and_seek_to_first(sstables: Vec<Arc<SsTable>>) -> Result<Self> {
        unimplemented!()
    }

    pub fn create_and_seek_to_key(sstables: Vec<Arc<SsTable>>, key: KeySlice) -> Result<Self> {
        unimplemented!()
    }
}

impl StorageIterator for SstConcatIterator {
    type KeyType<'a> = KeySlice<'a>;

    fn key(&self) -> Self::KeyType<'_> {
        unimplemented!()
    }

    fn value(&self) -> &[u8] {
        unimplemented!()
    }

    fn is_valid(&self) -> bool {
        unimplemented!()
    }

    fn next(&self) -> Result<()> {
        unimplemented!()
    }

    fn num_active_iterators(&self) -> usize {
        1
    }
}
