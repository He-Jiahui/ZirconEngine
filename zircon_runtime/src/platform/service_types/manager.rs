use std::sync::Arc;

use crate::core::framework::platform::{
    ApplicationLifecycleSnapshot, PlatformHostSnapshot, PreferenceEviction, PreferenceFlushTicket,
    PreferenceKey, PreferenceMutationSubmission, PreferenceReadSnapshot, PreferenceStorage,
    PreferenceStorageError, PreferenceWorkDeadline,
};
use crate::core::framework::window::DisplayTopologySnapshot;

use super::PlatformDriver;
use crate::platform::preferences::{InstalledPreferenceBackendKind, PreferencePersistenceAdapter};
use crate::platform::{PlatformCapabilityReport, PlatformConfig, PlatformRuntimeCapabilityReport};

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

    /// Returns the static catalog used for offline planning only. Product
    /// admission must consume `runtime_capability_report` instead.
    pub fn planning_capability_report(&self, config: &PlatformConfig) -> PlatformCapabilityReport {
        config.planning_capability_report_with_preference_storage_backend(self.backend_kind())
    }

    /// Projects catalog entries through the installed platform-host owner and
    /// its observed evidence. A compiled feature cannot become Ready without
    /// this runtime fact.
    pub fn runtime_capability_report(
        &self,
        config: &PlatformConfig,
    ) -> PlatformRuntimeCapabilityReport {
        PlatformRuntimeCapabilityReport::new(
            config.enabled,
            self.planning_capability_report(config),
            self.driver.platform_host_snapshot(),
        )
    }

    /// Returns the platform driver's immutable display facts. Commands remain
    /// brokered by the platform host; callers never receive a native monitor.
    pub fn display_topology_snapshot(&self) -> Arc<DisplayTopologySnapshot> {
        self.driver.display_topology_snapshot()
    }

    /// Publishes only immutable host state. Native backend objects remain
    /// private to the process host and commands stay on the driver path.
    pub fn platform_host_snapshot(&self) -> PlatformHostSnapshot {
        self.driver.platform_host_snapshot()
    }

    pub fn application_lifecycle_snapshot(&self) -> ApplicationLifecycleSnapshot {
        self.driver.application_lifecycle_snapshot()
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

#[cfg(test)]
mod tests;
