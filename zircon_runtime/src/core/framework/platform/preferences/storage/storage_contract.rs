use std::sync::Arc;

use super::super::{PreferenceKey, PreferenceStorageBackendKind, PreferenceStorageError};
use super::{
    snapshot::{PreferenceEviction, PreferenceReadSnapshot},
    tickets::{PreferenceFlushTicket, PreferenceMutationSubmission},
    work_deadline::PreferenceWorkDeadline,
};

/// Versioned manager contract consumed by runtime clients and host adapters.
pub trait PreferenceStorage: Send + Sync + 'static {
    fn backend_kind(&self) -> PreferenceStorageBackendKind;

    fn snapshot(
        &self,
        key: &PreferenceKey,
    ) -> Result<PreferenceReadSnapshot, PreferenceStorageError>;

    fn submit_write(
        &self,
        key: PreferenceKey,
        value: Arc<[u8]>,
        deadline: PreferenceWorkDeadline,
    ) -> Result<PreferenceMutationSubmission, PreferenceStorageError>;

    fn submit_remove(
        &self,
        key: PreferenceKey,
        deadline: PreferenceWorkDeadline,
    ) -> Result<PreferenceMutationSubmission, PreferenceStorageError>;

    fn flush_fence(
        &self,
        deadline: PreferenceWorkDeadline,
    ) -> Result<Arc<dyn PreferenceFlushTicket>, PreferenceStorageError>;

    /// Explicitly discards one terminal, visible-not-durable generation and its failure state.
    ///
    /// Pending and durable generations are not eligible for lossy eviction.
    fn evict(&self, key: &PreferenceKey) -> Option<PreferenceEviction>;
}
