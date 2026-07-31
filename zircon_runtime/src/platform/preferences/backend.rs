use std::io::Read;

use crate::core::framework::platform::{
    PreferenceKey, PreferenceStorageBackendKind, PreferenceStorageError,
};

use super::PreferenceBackendWorkAuthority;

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
}
