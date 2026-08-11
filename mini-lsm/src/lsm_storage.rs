use std::{
    collections::HashMap,
    ops::{Bound, RangeBounds},
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicUsize},
};

use anyhow::Result;
use bytes::Bytes;
use parking_lot::{Mutex, MutexGuard, RwLock};

use crate::{
    block::BlockCache,
    compact::{
        CompactionController, CompactionOptions, LeveledCompactionController,
        LeveledCompactionOptions, SimpleLeveledCompactionController,
        SimpleLeveledCompactionOptions, TieredCompactionController,
    },
    iterators::merge_iterator::MergeIterator,
    lsm_iterator::{FusedIterator, LsmIterator},
    manifest::Manifest,
    mem_table::{MemTable, MemTableIterator, map_bound},
    mvcc::LsmMvccInner,
    range::LsmRange,
    table::SsTable,
};

#[derive(Debug, Clone)]
pub struct LsmStorageOptions {
    pub block_size: usize,
    pub sst_size_target: usize,
    pub memtable_count_limit: usize,
    pub compaction_options: CompactionOptions,
    pub enable_wal: bool,
    pub serializable: bool,
}

impl LsmStorageOptions {
    pub fn default_for_week1_test() -> Self {
        Self {
            block_size: 4096,
            sst_size_target: 2 << 20,
            memtable_count_limit: 50,
            compaction_options: CompactionOptions::NoCompaction,
            enable_wal: false,
            serializable: false,
        }
    }

    pub fn default_for_week1_day6_test() -> Self {
        Self {
            block_size: 4096,
            sst_size_target: 2 << 20,
            memtable_count_limit: 2,
            compaction_options: CompactionOptions::NoCompaction,
            enable_wal: false,
            serializable: false,
        }
    }

    pub fn default_for_week2_test(compaction_options: CompactionOptions) -> Self {
        Self {
            block_size: 4096,
            sst_size_target: 2 << 20,
            memtable_count_limit: 2,
            compaction_options,
            enable_wal: false,
            serializable: false,
        }
    }
}

/// Represents the state of the storage engine.
#[derive(Clone)]
pub struct LsmStorageState {
    /// The current memtable.
    pub memtable: Arc<MemTable>,
    /// Immutable memtable, from latest to earliest.
    pub memtable_imm: Vec<Arc<MemTable>>,
    /// L0 SSTs, from latest to earlies.
    pub sstables_l0: Vec<usize>,
    /// SsTables sorted by key range; L1 - L_max for leveled compaction, or tiers for tiered
    /// compaction
    pub levels: Vec<(usize, Vec<usize>)>,
    /// SST object
    pub sstables: HashMap<usize, Arc<SsTable>>,
}

impl LsmStorageState {
    fn create(options: &LsmStorageOptions) -> Self {
        let levels = match &options.compaction_options {
            CompactionOptions::Leveled(LeveledCompactionOptions { max_levels, .. })
            | CompactionOptions::Simple(SimpleLeveledCompactionOptions { max_levels, .. }) => (1
                ..=*max_levels)
                .map(|level| (level, Vec::new()))
                .collect::<Vec<_>>(),
            CompactionOptions::Tiered(_) => Vec::new(),
            CompactionOptions::NoCompaction => vec![(1, Vec::new())],
        };
        Self {
            memtable: Arc::new(MemTable::create(0)),
            memtable_imm: Vec::new(),
            sstables_l0: Vec::new(),
            levels,
            sstables: Default::default(),
        }
    }
}

pub enum WriteBatchRecord<T>
where
    T: AsRef<[u8]>,
{
    Put(T, T),
    Del(T),
}

#[derive(Clone, Debug)]
pub enum CompactionFilter {
    Prefix(Bytes),
}

/// The storage interface of the LSM tree.
pub(crate) struct LsmStorageInner {
    pub(crate) state: Arc<RwLock<Arc<LsmStorageState>>>,
    pub(crate) state_lock: Mutex<()>,
    path: PathBuf,
    pub(crate) block_cache: Arc<BlockCache>,
    next_sst_id: AtomicUsize,
    pub(crate) options: Arc<LsmStorageOptions>,
    pub(crate) manifest: Option<Manifest>,
    pub(crate) mvcc: Option<LsmMvccInner>,
    pub(crate) compaction_controller: CompactionController,
    pub(crate) compaction_filters: Arc<Mutex<Vec<CompactionFilter>>>,
}

impl LsmStorageInner {
    /// Start the storage engine by either loading an existing directory or creating a new one if
    /// the directory does not exist.
    pub(crate) fn open(path: impl AsRef<Path>, options: LsmStorageOptions) -> Result<Self> {
        let path = path.as_ref();
        let state = LsmStorageState::create(&options);

        let compaction_controller = match &options.compaction_options {
            CompactionOptions::Leveled(options) => {
                CompactionController::Leveled(LeveledCompactionController::new(options.clone()))
            }
            CompactionOptions::Tiered(options) => {
                CompactionController::Tiered(TieredCompactionController::new(options.clone()))
            }
            CompactionOptions::Simple(options) => CompactionController::Simple(
                SimpleLeveledCompactionController::new(options.clone()),
            ),
            CompactionOptions::NoCompaction => CompactionController::NoCompaction,
        };

        Ok(Self {
            state: Arc::new(RwLock::new(Arc::new(state))),
            state_lock: Mutex::new(()),
            path: path.to_path_buf(),
            block_cache: Arc::new(BlockCache::new(1024)),
            next_sst_id: AtomicUsize::new(1),
            options: options.into(),
            manifest: None,
            mvcc: None,
            compaction_controller,
            compaction_filters: Arc::new(Mutex::new(Vec::new())),
        })
    }

    pub(crate) fn next_sst_id(&self) -> usize {
        self.next_sst_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
    }

    pub(crate) fn mvcc(&self) -> &LsmMvccInner {
        self.mvcc.as_ref().unwrap()
    }

    pub fn sync(&self) -> Result<()> {
        unimplemented!()
    }

    pub fn add_compaction_filter(&self, compaction_filter: CompactionFilter) {
        let mut compaction_filters = self.compaction_filters.lock();
        compaction_filters.push(compaction_filter)
    }

    fn snapshot_state(&self) -> Arc<LsmStorageState> {
        let guard = self.state.read();
        Arc::clone(&guard)
    }

    /// Get a key from the storage.
    pub fn get(&self, key: impl AsRef<[u8]>) -> Result<Option<Bytes>> {
        let key = key.as_ref();
        let snapshot = self.snapshot_state();

        let value = match snapshot.memtable.get(key.as_ref()) {
            Some(val) => Some(val),
            None => snapshot
                .memtable_imm
                .iter()
                .find_map(|memtable| memtable.get(key.as_ref())),
        };

        Ok(value.filter(|bytes| !bytes.is_empty()))
    }

    /// Put a key-value pair into the storage by writing into the current memtable.
    pub fn put(&self, key: impl AsRef<[u8]>, value: impl AsRef<[u8]>) -> Result<()> {
        let snapshot = self.snapshot_state();
        snapshot.memtable.put(key, value)?;

        if snapshot.memtable.approximate_size() >= self.options.sst_size_target {
            let state_lock = self.state_lock.lock();

            if self.snapshot_state().memtable.approximate_size() >= self.options.sst_size_target {
                return self.force_freeze_memtable(&state_lock);
            }
        }

        Ok(())
    }

    /// Remove a key from the storage by writing an empty value.
    pub fn delete<K>(&self, key: K) -> Result<()>
    where
        K: AsRef<[u8]>,
    {
        self.put(key, [])
    }

    pub fn scan_range(&self, range: impl LsmRange) -> Result<FusedIterator<LsmIterator>> {
        self.scan(range.start(), range.end())
    }

    /// Create an iterator over a range of keys.
    pub fn scan(
        &self,
        lower: Bound<&[u8]>,
        upper: Bound<&[u8]>,
    ) -> Result<FusedIterator<LsmIterator>> {
        let snapshot = self.snapshot_state();

        let active_iter = std::iter::once(Box::new(snapshot.memtable.scan(lower, upper)));
        let imm_iters = snapshot
            .memtable_imm
            .iter()
            .map(|memtable| Box::new(memtable.scan(lower, upper)));
        let mem_iters: Vec<Box<MemTableIterator>> = active_iter.chain(imm_iters).collect();

        let merged_iter = MergeIterator::create(mem_iters);
        let lsm_iter = LsmIterator::new(merged_iter)?;

        Ok(FusedIterator::new(lsm_iter))
    }

    /// Write a batch of data into the storage.
    pub fn write_batch<T>(&self, _batch: &[WriteBatchRecord<T>]) -> Result<()>
    where
        T: AsRef<[u8]>,
    {
        unimplemented!()
    }

    pub(crate) fn sst_path_static<P>(path: P, id: usize) -> PathBuf
    where
        P: AsRef<Path>,
    {
        path.as_ref().join(format!("{:05}.sst", id))
    }

    pub(crate) fn sst_path(&self, id: usize) -> PathBuf {
        Self::sst_path_static(&self.path, id)
    }

    pub(crate) fn wal_path_static<P>(path: P, id: usize) -> PathBuf
    where
        P: AsRef<Path>,
    {
        path.as_ref().join(format!("{:05}.wal", id))
    }

    pub(crate) fn wal_path(&self, id: usize) -> PathBuf {
        Self::wal_path_static(&self.path, id)
    }

    pub(super) fn sync_dir(&self) -> Result<()> {
        unimplemented!()
    }

    /// Force freeze the current memtable to an immutable memtable.
    pub fn force_freeze_memtable(&self, _state_lock_observer: &MutexGuard<'_, ()>) -> Result<()> {
        let id = self.next_sst_id();
        let memtable = Arc::new(MemTable::create_with_wal(id, self.wal_path(id))?);
        {
            let mut guard = self.state.write();
            let mut snapshot = guard.as_ref().clone();
            let memtable_old = std::mem::replace(&mut snapshot.memtable, memtable);
            snapshot.memtable_imm.insert(0, memtable_old);
            *guard = Arc::new(snapshot);
        }
        Ok(())
    }

    /// Force flush the earliest-created immutable memtable to disk
    pub fn force_flush_next_memtable_imm(&self) -> Result<()> {
        unimplemented!()
    }

    pub fn new_txn(&self) -> Result<()> {
        // no-op
        Ok(())
    }
}

/// A thin wrapper for `LsmStorageInner` and the user interface for MiniLSM.
pub struct MiniLsm {
    pub(crate) inner: Arc<LsmStorageInner>,
    /// Notifies the compaction thread to stop working.
    compaction_notifier: crossbeam_channel::Sender<()>,
    /// The handle for the compaction thread.
    compaction_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
    /// Notifies the L0 flush thread to stop working.
    flush_notifier: crossbeam_channel::Sender<()>,
    /// The handle for the flush thread.
    flush_thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}

impl Drop for MiniLsm {
    fn drop(&mut self) {
        self.compaction_notifier.send(()).ok();
        self.flush_notifier.send(()).ok();
    }
}

impl MiniLsm {
    /// Start the storge engine bye either loading an existing directory or creating a new one if
    /// the directory does not exist.
    pub fn open<P>(path: P, options: LsmStorageOptions) -> Result<Arc<Self>>
    where
        P: AsRef<Path>,
    {
        let inner = Arc::new(LsmStorageInner::open(path, options)?);
        let (tx1, rx) = crossbeam_channel::unbounded();
        let compaction_thread = inner.spawn_compaction_thread(rx)?;
        let (tx2, rx) = crossbeam_channel::unbounded();
        let flush_thread = inner.spawn_flush_thread(rx)?;
        Ok(Arc::new(Self {
            inner,
            compaction_notifier: tx1,
            compaction_thread: Mutex::new(compaction_thread),
            flush_notifier: tx2,
            flush_thread: Mutex::new(flush_thread),
        }))
    }

    pub fn close(&self) -> Result<()> {
        unimplemented!()
    }

    pub fn new_txn(&self) -> Result<()> {
        self.inner.new_txn()
    }

    pub fn write_batch<T>(&self, batch: &[WriteBatchRecord<T>]) -> Result<()>
    where
        T: AsRef<[u8]>,
    {
        self.inner.write_batch(batch)
    }

    pub fn add_compaction_filter(&self, compaction_filter: CompactionFilter) {
        self.inner.add_compaction_filter(compaction_filter);
    }

    pub fn get(&self, key: &[u8]) -> Result<Option<Bytes>> {
        self.inner.get(key)
    }

    pub fn put(&self, key: &[u8], value: &[u8]) -> Result<()> {
        self.inner.put(key, value)
    }

    pub fn delete(&self, key: &[u8]) -> Result<()> {
        self.inner.delete(key)
    }

    pub fn sync(&self) -> Result<()> {
        self.inner.sync()
    }

    pub fn scan_range(&self, range: impl LsmRange) -> Result<FusedIterator<LsmIterator>> {
        self.inner.scan_range(range)
    }

    pub fn scan(
        &self,
        lower: Bound<&[u8]>,
        upper: Bound<&[u8]>,
    ) -> Result<FusedIterator<LsmIterator>> {
        self.inner.scan(lower, upper)
    }

    pub fn force_flush(&self) -> Result<()> {
        if !self.inner.state.read().memtable.is_empty() {
            self.inner
                .force_freeze_memtable(&self.inner.state_lock.lock())?;
        }

        if !self.inner.state.read().memtable_imm.is_empty() {
            self.inner.force_flush_next_memtable_imm()?;
        }

        Ok(())
    }

    pub fn force_full_compaction(&self) -> Result<()> {
        self.inner.force_full_compaction()
    }
}
