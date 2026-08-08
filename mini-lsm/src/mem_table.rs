use std::{
    ops::Bound,
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use anyhow::Result;
use bytes::Bytes;
use crossbeam_skiplist::SkipMap;
use ouroboros::self_referencing;

use crate::{iterators::StorageIterator, key::KeySlice, table::SsTableBuilder, wal::Wal};

/// A basic mem-table based on crossbeam-skiplist.
pub struct MemTable {
    map: Arc<SkipMap<Bytes, Bytes>>,
    wal: Option<Wal>,
    id: usize,
    approximate_size: Arc<AtomicUsize>,
}

/// Create a bound of `Bytes` from a bound of `&[u8]`.
pub(crate) fn map_bound(bound: Bound<&[u8]>) -> Bound<Bytes> {
    match bound {
        Bound::Included(x) => Bound::Included(Bytes::copy_from_slice(x)),
        Bound::Excluded(x) => Bound::Excluded(Bytes::copy_from_slice(x)),
        Bound::Unbounded => Bound::Unbounded,
    }
}

impl MemTable {
    /// Create a new mem-table
    pub fn create(id: usize) -> Self {
        Self {
            map: Arc::new(SkipMap::new()),
            wal: None,
            id,
            approximate_size: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Create a new mem-table with WAL
    pub fn create_with_wal(id: usize, path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            map: Arc::new(SkipMap::new()),
            wal: Some(Wal::create(path)?),
            id,
            approximate_size: Arc::new(AtomicUsize::new(0)),
        })
    }

    /// Create a mem-table with WAL
    pub fn recover_from_wal<P>(_id: usize, _path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        unimplemented!()
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn approximate_size(&self) -> usize {
        self.approximate_size.load(Ordering::Relaxed)
    }

    /// Only use this function when closing the database
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Get a value by key.
    pub fn get(&self, key: impl AsRef<[u8]>) -> Option<Bytes> {
        self.map
            .get(key.as_ref())
            .map(|entry| entry.value().clone())
    }

    /// Put a key-value pair into the mem-table
    pub fn put(&self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        let key = key.as_ref();
        let value = value.as_ref();
        self.map
            .insert(Bytes::copy_from_slice(key), Bytes::copy_from_slice(value));
        self.approximate_size
            .fetch_add(key.len() + value.len(), Ordering::Relaxed);
        Ok(())
    }

    pub fn put_batch(&self, data: &[(KeySlice, &[u8])]) -> Result<()> {
        unimplemented!()
    }

    pub fn sync_wal(&self) -> Result<()> {
        if let Some(ref wal) = self.wal {
            wal.sync()?;
        }
        Ok(())
    }

    /// Get an iterator over a range of keys.
    pub fn scan(&self, lower: Bound<&[u8]>, upper: Bound<&[u8]>) -> MemTableIterator {
        unimplemented!()
    }

    /// Flush the mem-table to SSTable.
    pub fn flush(&self, builder: &mut SsTableBuilder) -> Result<()> {
        unimplemented!()
    }

    pub fn for_testing_put_slice(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.put(key, value)
    }

    pub fn for_testing_get_slice(&self, key: &[u8]) -> Option<Bytes> {
        self.get(key)
    }

    pub fn for_testing_scan_slice(
        &self,
        lower: Bound<&[u8]>,
        upper: Bound<&[u8]>,
    ) -> MemTableIterator {
        self.scan(lower, upper)
    }
}

type SkipMapRangeIter<'a> =
    crossbeam_skiplist::map::Range<'a, Bytes, (Bound<Bytes>, Bound<Bytes>), Bytes, Bytes>;

/// An iterator over a range of `SkipMap`.
#[self_referencing]
pub struct MemTableIterator {
    /// Stores a referene to the skipmap.
    map: Arc<SkipMap<Bytes, Bytes>>,
    /// Stores a skipmap iterator that refers to the lifetime of `MemTableIterator` itself.
    #[borrows(map)]
    #[not_covariant]
    iter: SkipMapRangeIter<'this>,
    /// Stores the current key-value pair.
    item: (Bytes, Bytes),
}

impl StorageIterator for MemTableIterator {
    type KeyType<'a> = KeySlice<'a>;

    fn next(&mut self) -> Result<()> {
        unimplemented!()
    }

    fn key(&self) -> Self::KeyType<'_> {
        unimplemented!()
    }

    fn value(&self) -> &[u8] {
        unimplemented!()
    }

    fn is_valid(&self) -> bool {
        unimplemented!()
    }
}
