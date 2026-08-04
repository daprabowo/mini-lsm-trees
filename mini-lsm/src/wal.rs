use std::{fs::File, io::BufWriter, path::Path, sync::Arc};

use anyhow::Result;
use bytes::Bytes;
use crossbeam_skiplist::SkipMap;
use parking_lot::Mutex;

use crate::key::KeySlice;

pub struct Wal {
    file: Arc<Mutex<BufWriter<File>>>,
}

impl Wal {
    pub fn create<P>(_path: P) -> Result<Self>
    where
        P: AsRef<Path>,
    {
        unimplemented!()
    }

    pub fn recover(_path: impl AsRef<Path>, _skiplist: &SkipMap<Bytes, Bytes>) -> Result<Self> {
        unimplemented!()
    }

    pub fn put(&self, _key: &[u8], _value: &[u8]) -> Result<()> {
        unimplemented!()
    }

    pub fn put_batch(&self, _data: &[(KeySlice, &[u8])]) -> Result<()> {
        unimplemented!()
    }

    pub fn sync(&self) -> Result<()> {
        unimplemented!()
    }
}
