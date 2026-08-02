use std::sync::Arc;

use crate::core::framework::platform::{
    PreferenceEviction, PreferenceFlushTicket, PreferenceKey, PreferenceMutationSubmission,
    PreferenceReadSnapshot, PreferenceStorage, PreferenceStorageError, PreferenceWorkDeadline,
};

use super::PlatformDriver;
use crate::platform::preferences::{InstalledPreferenceBackendKind, PreferencePersistenceAdapter};
use crate::platform::{PlatformCapabilityReport, PlatformConfig};

#[derive(Clone, Debug)]
pub struct PlatformManager {
    driver: Arc<PlatformDriver>,
    preferences: Arc<PreferencePersistenceAdapter>,
}

impl PlatformManager {
    pub(crate) fn new(driver: Arc<PlatformDriver>) -> Self {
        Self {
            preferences: driver.preference_persistence_adapter(),
            driver,
        }
    }

    pub fn capability_report(&self, config: &PlatformConfig) -> PlatformCapabilityReport {
        config.capability_report_with_preference_storage_backend(self.backend_kind())
    }
}

impl PreferenceStorage for PlatformManager {
    fn backend_kind(&self) -> InstalledPreferenceBackendKind {
        self.driver.preference_storage_backend_kind()
    }

    fn snapshot(
        &self,
        key: &PreferenceKey,
    ) -> Result<PreferenceReadSnapshot, PreferenceStorageError> {
        self.preferences.snapshot(key)
    }

    fn submit_write(
        &self,
        key: PreferenceKey,
        value: Arc<[u8]>,
        deadline: PreferenceWorkDeadline,
    ) -> Result<PreferenceMutationSubmission, PreferenceStorageError> {
        self.preferences.submit_write(key, value, deadline)
    }

    fn submit_remove(
        &self,
        key: PreferenceKey,
        deadline: PreferenceWorkDeadline,
    ) -> Result<PreferenceMutationSubmission, PreferenceStorageError> {
        self.preferences.submit_remove(key, deadline)
    }

    fn flush_fence(
        &self,
        deadline: PreferenceWorkDeadline,
    ) -> Result<Arc<dyn PreferenceFlushTicket>, PreferenceStorageError> {
        self.preferences.flush_fence(deadline)
    }

    fn evict(&self, key: &PreferenceKey) -> Option<PreferenceEviction> {
        self.preferences.evict(key)
    }
}
