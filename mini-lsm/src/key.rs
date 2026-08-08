use std::fmt::Debug;

use bytes::Bytes;

pub const TS_ENABLED: bool = false;

pub struct Key<T>(T)
where
    T: AsRef<[u8]>;

pub type KeySlice<'a> = Key<&'a [u8]>;
pub type KeyVec = Key<Vec<u8>>;
pub type KeyBytes = Key<Bytes>;

impl<T> Key<T>
where
    T: AsRef<[u8]>,
{
    pub fn into_inner(self) -> T {
        self.0
    }

    pub fn len(&self) -> usize {
        self.0.as_ref().len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.as_ref().is_empty()
    }

    pub fn for_testing_ts(self) -> u64 {
        0
    }
}

impl Key<Vec<u8>> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    /// Create a `KeyVec` from a `Vec<u8>`.
    pub fn from_vec(key: Vec<u8>) -> Self {
        Self(key)
    }

    /// Clears the key and set ts to 0.
    pub fn clear(&mut self) {
        self.0.clear();
    }

    /// Append a slice to the end of the key.
    pub fn append(&mut self, data: &[u8]) {
        self.0.extend(data);
    }

    /// Set the key from a slice without re-allocating.
    pub fn set_from_slice(&mut self, key_slice: KeySlice) {
        self.0.clear();
        self.0.extend(key_slice.0);
    }

    pub fn as_key_slice(&self) -> KeySlice<'_> {
        Key(self.0.as_slice())
    }

    pub fn into_key_bytes(self) -> KeyBytes {
        Key(self.0.into())
    }

    pub fn raw_ref(&self) -> &[u8] {
        self.0.as_ref()
    }

    pub fn for_testing_key_ref(&self) -> &[u8] {
        self.0.as_ref()
    }

    pub fn for_testing_from_vec_no_ts(key: Vec<u8>) -> Self {
        Self(key)
    }
}

impl Key<Bytes> {
    /// Create a `KeyBytes` from a `Bytes`.
    pub fn from_bytes(bytes: Bytes) -> Self {
        Self(bytes)
    }

    pub fn as_key_slice(&self) -> KeySlice<'_> {
        Key(&self.0)
    }

    pub fn raw_ref(&self) -> &[u8] {
        self.0.as_ref()
    }

    pub fn for_testing_key_ref(&self) -> &[u8] {
        self.0.as_ref()
    }

    pub fn for_testing_from_bytes_no_ts(bytes: Bytes) -> Self {
        Self(bytes)
    }
}

impl<'a> Key<&'a [u8]> {
    /// Create a key slice from a slice.
    pub fn from_slice(slice: &'a [u8]) -> Self {
        Self(slice)
    }

    pub fn to_key_vec(self) -> KeyVec {
        Key(self.0.to_vec())
    }

    pub fn raw_ref(&self) -> &[u8] {
        self.0
    }

    pub fn for_testing_key_ref(&self) -> &[u8] {
        self.0
    }

    pub fn for_testing_from_slice_no_ts(slice: &'a [u8]) -> Self {
        Self(slice)
    }

    pub fn for_testing_from_slice_with_ts(slice: &'a [u8], _ts: u64) -> Self {
        Self(slice)
    }
}

impl<T> Debug for Key<T>
where
    T: AsRef<[u8]> + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<T> Default for Key<T>
where
    T: AsRef<[u8]> + Default,
{
    fn default() -> Self {
        Self(T::default())
    }
}

impl<T> Clone for Key<T>
where
    T: AsRef<[u8]> + Clone,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Copy for Key<T> where T: AsRef<[u8]> + Copy {}

impl<T> PartialEq for Key<T>
where
    T: AsRef<[u8]> + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(&other.0)
    }
}

impl<T> Eq for Key<T> where T: AsRef<[u8]> + Eq {}

impl<T> PartialOrd for Key<T>
where
    T: AsRef<[u8]> + PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl<T> Ord for Key<T>
where
    T: AsRef<[u8]> + Ord,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
