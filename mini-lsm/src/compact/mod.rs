mod leveled;
mod simple_leveled;
mod tiered;

pub use leveled::{LeveledCompactionController, LeveledCompactionOptions, LeveledCompactionTask};
pub use simple_leveled::{
    SimpleLeveledCompactionController, SimpleLeveledCompactionOptions, SimpleLeveledCompactionTask,
};
pub use tiered::{TieredCompactionController, TieredCompactionOptions, TieredCompactionTask};

use std::{sync::Arc, time::Duration};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    iterators::StorageIterator,
    key::KeySlice,
    lsm_storage::{LsmStorageInner, LsmStorageState},
    table::{SsTable, SsTableBuilder},
};

#[derive(Debug, Serialize, Deserialize)]
pub enum CompactionTask {
    Leveled(LeveledCompactionTask),
    Tiered(TieredCompactionTask),
    Simple(SimpleLeveledCompactionTask),
    ForceFullCompaction {
        l0_sstables: Vec<usize>,
        l1_sstables: Vec<usize>,
    },
}

impl CompactionTask {
    fn compact_to_bottom_level(&self) -> bool {
        match self {
            Self::Leveled(task) => task.is_lower_level_botom_level,
            Self::Simple(task) => task.is_lower_level_botom_level,
            Self::Tiered(task) => task.bottom_tier_included,
            Self::ForceFullCompaction { .. } => true,
        }
    }
}

pub(crate) enum CompactionController {
    Leveled(LeveledCompactionController),
    Tiered(TieredCompactionController),
    Simple(SimpleLeveledCompactionController),
    NoCompaction,
}

impl CompactionController {
    /// Generates a compaction task.
    ///
    /// Returns `None` if no compaction needs to be scheduled. The order of SSTs in the compaction
    /// task id vector matters.
    pub fn generate_compaction_task(&self, snapshot: &LsmStorageState) -> Option<CompactionTask> {
        match self {
            Self::Leveled(controller) => controller
                .generate_compaction_task(snapshot)
                .map(CompactionTask::Leveled),
            Self::Tiered(controller) => controller
                .generate_compaction_task(snapshot)
                .map(CompactionTask::Tiered),
            Self::Simple(controller) => controller
                .generate_compaction_task(snapshot)
                .map(CompactionTask::Simple),
            Self::NoCompaction => unreachable!(),
        }
    }

    /// Apply the compaction result.
    ///
    /// The compactor will call this function with the compaction task and the list of SST ids
    /// generated. This function applies the result and generates a new LSM state. The functions
    /// should only change `l0_sstables` and `levels` without changing memtables and `sstables`
    /// hashmap. Though there should only be one thread running compaction jobs, you should think
    /// about the case where an L0 SST gets flushed while the compactor generates new SSTs, and with
    /// that in mind, you should do some sanity checks in your implementation.
    pub fn apply_compaction_result(
        &self,
        snapshot: &LsmStorageState,
        task: &CompactionTask,
        output: &[usize],
        in_recovery: bool,
    ) -> (LsmStorageState, Vec<usize>) {
        match (self, task) {
            (Self::Leveled(controller), CompactionTask::Leveled(task)) => {
                controller.apply_compaction_result(snapshot, task, output, in_recovery)
            }
            (Self::Tiered(controller), CompactionTask::Tiered(task)) => {
                controller.apply_compaction_result(snapshot, task, output, in_recovery)
            }
            (Self::Simple(controller), CompactionTask::Simple(task)) => {
                controller.apply_compaction_result(snapshot, task, output, in_recovery)
            }
            _ => unreachable!(),
        }
    }
}

impl CompactionController {
    pub fn flush_to_l0(&self) -> bool {
        matches!(
            self,
            Self::Leveled(_) | Self::Simple(_) | Self::NoCompaction
        )
    }
}

#[derive(Debug, Clone)]
pub enum CompactionOptions {
    /// Leveled compaction with partial compaction + dynamic level support (= RocksDB's leveled
    /// Compaction).
    Leveled(LeveledCompactionOptions),
    /// Tiered compaction (= RocksDB's universal compaction).
    Tiered(TieredCompactionOptions),
    /// Simple leveled compaction.
    Simple(SimpleLeveledCompactionOptions),
    /// In no compaction.
    NoCompaction,
}

impl LsmStorageInner {
    fn compact(&self, _task: &CompactionTask) -> Result<Vec<Arc<SsTable>>> {
        unimplemented!()
    }

    pub fn force_full_compaction(&self) -> Result<()> {
        unimplemented!()
    }

    fn trigger_compaction(&self) -> Result<()> {
        unimplemented!()
    }

    pub(crate) fn spawn_compaction_thread(
        self: &Arc<Self>,
        rx: crossbeam_channel::Receiver<()>,
    ) -> Result<Option<std::thread::JoinHandle<()>>> {
        if let CompactionOptions::Leveled(_)
        | CompactionOptions::Simple(_)
        | CompactionOptions::Tiered(_) = self.options.compaction_options
        {
            let this = self.clone();
            let handle = std::thread::spawn(move || {
                let ticker = crossbeam_channel::tick(Duration::from_millis(50));
                loop {
                    crossbeam_channel::select! {
                        recv(ticker) -> _ => if let Err(e) = this.trigger_compaction() {
                            eprintln!("compaction failed: {}", e);
                        },
                        recv(rx) -> _ => return
                    }
                }
            });
            return Ok(Some(handle));
        }
        Ok(None)
    }

    fn trigger_flush(&self) -> Result<()> {
        Ok(())
    }

    pub(crate) fn spawn_flush_thread(
        self: &Arc<Self>,
        rx: crossbeam_channel::Receiver<()>,
    ) -> Result<Option<std::thread::JoinHandle<()>>> {
        let this = self.clone();
        let handle = std::thread::spawn(move || {
            let ticker = crossbeam_channel::tick(Duration::from_millis(50));
            loop {
                crossbeam_channel::select! {
                    recv(ticker) -> _ => if let Err(e) = this.trigger_flush() {
                        eprintln!("flush failed: {}", e);
                    },
                    recv(rx) -> _ => return
                }
            }
        });
        Ok(Some(handle))
    }
}
