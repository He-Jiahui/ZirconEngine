use std::io::Read;
use std::time::Duration;

use crate::core::framework::platform::{
    PreferenceKey, PreferenceStorageBackendKind, PreferenceStorageError,
};

use super::PreferenceBackendWorkAuthority;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PreferenceStorageBackendDiagnostics {
    pub path_build_wall: Duration,
    pub path_cache_hits: u64,
    pub path_cache_misses: u64,
    pub path_builds: u64,
    pub path_cache_evictions: u64,
    pub path_cache_entries: u64,
    pub staged_write_wall: Duration,
    pub fsync_wall: Duration,
    pub reads: u64,
    pub writes: u64,
    pub removes: u64,
    pub flushes: u64,
}

/// Host-owned persistence implementation. Primitive access requires worker authority.
pub trait PreferenceStorageBackend: Send + Sync + 'static {
    fn backend_kind(&self) -> PreferenceStorageBackendKind;

    fn open_read(
        &self,
        authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
    ) -> Result<Option<Box<dyn Read + Send>>, PreferenceStorageError>;

    fn write(
        &self,
        authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
        value: &[u8],
    ) -> Result<(), PreferenceStorageError>;

    fn remove(
        &self,
        authority: &PreferenceBackendWorkAuthority,
        key: &PreferenceKey,
    ) -> Result<(), PreferenceStorageError>;

    fn flush(
        &self,
        authority: &PreferenceBackendWorkAuthority,
    ) -> Result<(), PreferenceStorageError>;

    fn diagnostics(&self) -> PreferenceStorageBackendDiagnostics {
        PreferenceStorageBackendDiagnostics::default()
    }
}
