use std::{path::Path, sync::Arc};

use anyhow::{Result, anyhow};
use bytes::{BufMut, Bytes};

use crate::{
    block::{BlockCache, builder::BlockBuilder},
    key::{KeyBytes, KeySlice},
    table::{BlockMeta, FileObject, SsTable},
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
        Self {
            builder: BlockBuilder::new(block_size),
            first_key: Vec::new(),
            last_key: Vec::new(),
            data: Vec::new(),
            meta: Vec::new(),
            block_size,
        }
    }

    /// Adds a key-value pair to SSTable.
    pub fn add(&mut self, key: KeySlice, value: &[u8]) {
        if self.builder.is_empty() {
            self.first_key = key.raw_ref().to_vec();
        }

        if self.builder.add(key, value) {
            self.last_key = key.raw_ref().to_vec();
            return;
        }

        let first_key = std::mem::replace(&mut self.first_key, key.raw_ref().to_vec());
        let last_key = std::mem::replace(&mut self.last_key, key.raw_ref().to_vec());
        let block_meta = BlockMeta {
            offset: self.data.len(),
            first_key: KeyBytes::from_vec(first_key),
            last_key: KeyBytes::from_vec(last_key),
        };
        self.meta.push(block_meta);

        let builder = std::mem::replace(&mut self.builder, BlockBuilder::new(self.block_size));
        let block_bytes = builder.build().encode();
        self.data.extend_from_slice(&block_bytes);

        self.builder.add(key, value);
    }

    /// Get the estimated size of the SSTable.
    ///
    /// Since the data blocks contain much more data than meta blocks, just return the size of data
    /// blocks here
    pub fn estimated_size(&self) -> usize {
        self.data.len()
    }

    /// Builds the SSTable and writes it to the given path.
    /// Use the `FileObject` structure to manipulate the disk objects.
    pub fn build(
        #[allow(unused_mut)] mut self,
        id: usize,
        block_cache: Option<Arc<BlockCache>>,
        path: impl AsRef<Path>,
    ) -> Result<SsTable> {
        let estimated_size = self.estimated_size();

        let last_key = KeyBytes::from_vec(self.last_key);
        let meta = BlockMeta {
            offset: self.data.len(),
            first_key: KeyBytes::from_vec(self.first_key),
            last_key: last_key.clone(),
        };
        self.meta.push(meta);
        let block = self.builder.build();
        let block_bytes = block.encode();
        self.data.extend_from_slice(&block_bytes);

        let block_meta_offset = self.data.len();

        let mut buf = Vec::with_capacity(estimated_size);
        buf.extend_from_slice(&self.data);
        BlockMeta::encode_block_meta(&self.meta, &mut buf);
        buf.put_u32(block_meta_offset as u32);

        let file = FileObject::create(path, buf)?;
        let first_key = self
            .meta
            .first()
            .and_then(|m| Some(m.first_key.clone()))
            .ok_or_else(|| anyhow!("empty builder"))?;

        Ok(SsTable {
            file,
            block_meta: self.meta,
            block_meta_offset,
            id,
            block_cache,
            first_key,
            last_key: last_key.clone(),
            bloom: None,
            max_ts: 0,
        })
    }

    #[cfg(test)]
    pub(crate) fn build_for_test(self, path: impl AsRef<Path>) -> Result<SsTable> {
        self.build(0, None, path)
    }
}
