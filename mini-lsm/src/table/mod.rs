pub(crate) mod bloom;
mod builder;
mod iterator;

pub use builder::SsTableBuilder;
pub use iterator::SsTableIterator;

use std::{fs::File, os::unix::fs::FileExt, path::Path, sync::Arc};

use anyhow::Result;
use bytes::{Buf, BufMut};

use crate::{
    block::{Block, BlockCache},
    key::{KeyBytes, KeySlice},
    table::bloom::Bloom,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BlockMeta {
    /// Offset of this data block.
    pub offset: usize,
    /// The first key of the data block.
    pub first_key: KeyBytes,
    /// The last key of the data block.
    pub last_key: KeyBytes,
}

impl BlockMeta {
    /// Encode block meta to a buffer.
    pub fn encode_block_meta(block_meta: &[BlockMeta], buf: &mut Vec<u8>) {
        if block_meta.is_empty() {
            return;
        }

        let sample_meta = &block_meta[0];
        let estimated_per_block = std::mem::size_of::<u32>()
            + std::mem::size_of::<u32>()
            + std::mem::size_of::<u16>()
            + sample_meta.first_key.raw_ref().len()
            + std::mem::size_of::<u16>()
            + sample_meta.last_key.raw_ref().len();
        buf.reserve(estimated_per_block * block_meta.len());
        buf.put_u32(block_meta.len() as u32);

        for meta in block_meta {
            let first_key_len = meta.first_key.len();
            let last_key_len = meta.last_key.len();

            let required_space = std::mem::size_of::<u32>()
                + std::mem::size_of::<u16>()
                + first_key_len
                + std::mem::size_of::<u16>()
                + last_key_len;
            buf.reserve(required_space);

            buf.put_u32(meta.offset as u32);
            buf.put_u16(meta.first_key.raw_ref().len() as u16);
            buf.extend_from_slice(meta.first_key.raw_ref());
            buf.put_u16(meta.last_key.raw_ref().len() as u16);
            buf.extend_from_slice(meta.last_key.raw_ref());
        }
    }

    /// Decode block meta from a buffer
    pub fn decode_block_meta(buf: impl Buf) -> Vec<BlockMeta> {
        let mut buf = buf;

        let block_count = buf.get_u32() as usize;
        let mut block_meta = Vec::with_capacity(block_count);

        while buf.has_remaining() {
            let offset = buf.get_u32() as usize;
            let first_key_len = buf.get_u16() as usize;
            let first_key = KeyBytes::from_bytes(buf.copy_to_bytes(first_key_len));
            let last_key_len = buf.get_u16() as usize;
            let last_key = KeyBytes::from_bytes(buf.copy_to_bytes(first_key_len));
            block_meta.push(BlockMeta {
                offset,
                first_key,
                last_key,
            });
        }

        block_meta
    }
}

pub struct FileObject(Option<File>, u64);

impl FileObject {
    pub fn read(&self, offset: u64, len: u64) -> Result<Vec<u8>> {
        let mut data = vec![0; len as usize];
        self.0
            .as_ref()
            .unwrap()
            .read_exact_at(&mut data[..], offset)?;
        Ok(data)
    }

    pub fn size(&self) -> u64 {
        self.1
    }

    /// Create a new file object and write the file to the disk.
    pub fn create(path: impl AsRef<Path>, data: Vec<u8>) -> Result<Self> {
        let path = path.as_ref();
        std::fs::write(path, &data)?;
        File::open(path)?.sync_all()?;
        Ok(FileObject(
            Some(File::options().read(true).write(true).open(path)?),
            data.len() as u64,
        ))
    }

    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let file = File::options().read(true).write(true).open(path)?;
        let size = file.metadata()?.len();
        Ok(FileObject(Some(file), size))
    }
}

pub struct SsTable {
    /// The actual storage unit of SSTable.
    pub(crate) file: FileObject,
    /// The meta blocks that hold info for data blocks.
    pub(crate) block_meta: Vec<BlockMeta>,
    /// The offset that indicates the start point of meta blocks in `file`.
    pub(crate) block_meta_offset: usize,
    id: usize,
    block_cache: Option<Arc<BlockCache>>,
    first_key: KeyBytes,
    last_key: KeyBytes,
    pub(crate) bloom: Option<Bloom>,
    /// The maximum timestamp stored in this SsTable.
    max_ts: u64,
}

impl SsTable {
    #[cfg(test)]
    pub(crate) fn open_for_test(file: FileObject) -> Result<Self> {
        Self::open(0, None, file)
    }

    /// Open SSTable from a file.
    pub fn open(id: usize, block_cache: Option<Arc<BlockCache>>, file: FileObject) -> Result<Self> {
        unimplemented!()
    }

    /// Create a mock SST with only first key + last key metadata.
    pub fn create_meta_only(
        id: usize,
        file_size: u64,
        first_key: KeyBytes,
        last_key: KeyBytes,
    ) -> Self {
        Self {
            file: FileObject(None, file_size),
            block_meta: vec![],
            block_meta_offset: 0,
            id,
            block_cache: None,
            first_key,
            last_key,
            bloom: None,
            max_ts: 0,
        }
    }

    /// Read a block from the disk.
    pub fn read_block(&self, block_idx: usize) -> Result<Arc<Block>> {
        unimplemented!()
    }

    /// Read a block from disk, with block cache.
    pub fn read_block_cached(&self, block_idx: usize) -> Result<Arc<Block>> {
        unimplemented!()
    }

    /// Find the block that may contain `key`.
    pub fn find_block_idx(&self, key: KeySlice) -> usize {
        // NOTE: You may want to make use of the `first_key` stored in `BlockMeta`.
        // You may also assume the key-value pairs stored in each consecutive block are sorted.
        unimplemented!()
    }

    /// Get number of data blocks.
    pub fn num_of_blocks(&self) -> usize {
        self.block_meta.len()
    }

    pub fn first_key(&self) -> &KeyBytes {
        &self.first_key
    }

    pub fn last_key(&self) -> &KeyBytes {
        &self.last_key
    }

    pub fn table_size(&self) -> u64 {
        self.file.1
    }

    pub fn sst_id(&self) -> usize {
        self.id
    }

    pub fn max_ts(&self) -> u64 {
        self.max_ts
    }
}
