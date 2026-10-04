use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

/// Process-local hash used only to select cache buckets.
///
/// Cache owners must still compare their full key and, for source-backed entries, the exact source
/// bytes. This type deliberately has no serialization or byte-export contract.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct EphemeralCacheHash(u64);

impl EphemeralCacheHash {
    pub(crate) fn from_hashable<T>(value: &T) -> Self
    where
        T: Hash + ?Sized,
    {
        let mut hasher = EphemeralCacheHasher::new();
        hasher.write(value);
        hasher.finish()
    }

    pub(crate) const fn from_process_hash(value: u64) -> Self {
        Self(value)
    }
}

pub(crate) struct EphemeralCacheHasher(DefaultHasher);

impl EphemeralCacheHasher {
    pub(crate) fn new() -> Self {
        Self(DefaultHasher::new())
    }

    pub(crate) fn write<T>(&mut self, value: &T)
    where
        T: Hash + ?Sized,
    {
        value.hash(&mut self.0);
    }

    pub(crate) fn finish(self) -> EphemeralCacheHash {
        EphemeralCacheHash(self.0.finish())
    }
}

/// Deterministic 32-byte identity stored at an artifact boundary.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub(crate) struct StableContentDigest([u8; 32]);

impl StableContentDigest {
    pub(crate) fn blake3(bytes: &[u8]) -> Self {
        Self(*blake3::hash(bytes).as_bytes())
    }

    pub(crate) const fn from_bytes(bytes: [u8; 32]) -> Self {
        Self(bytes)
    }

    pub(crate) const fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub(crate) const fn into_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl From<[u8; 32]> for StableContentDigest {
    fn from(value: [u8; 32]) -> Self {
        Self::from_bytes(value)
    }
}

impl From<StableContentDigest> for [u8; 32] {
    fn from(value: StableContentDigest) -> Self {
        value.into_bytes()
    }
}

#[cfg(test)]
#[path = "tests/identity.rs"]
mod tests;
