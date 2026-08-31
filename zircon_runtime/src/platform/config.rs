use serde::{Deserialize, Serialize};

use crate::core::framework::platform::{PreferenceStorageBackendKind, RuntimeTargetMode};

use super::{
    PlatformCapabilityMatrix, PlatformCapabilityReport, PlatformFeatureSelection, PlatformTarget,
};

pub const PLATFORM_CONFIG_KEY: &str = "runtime.platform.config";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlatformConfig {
    pub enabled: bool,
    pub target: PlatformTarget,
    pub target_mode: RuntimeTargetMode,
    pub features: PlatformFeatureSelection,
}

impl PlatformConfig {
    /// Returns the static compile/target catalog used for planning. It does
    /// not prove that a platform host is installed or observed at runtime.
    pub fn planning_capability_report(&self) -> PlatformCapabilityReport {
        let report =
            PlatformCapabilityMatrix::new(self.features).report(self.target, self.target_mode);
        if self.enabled {
            report
        } else {
            report.disabled_by_platform()
        }
    }

    pub fn planning_capability_report_with_preference_storage_backend(
        &self,
        backend: PreferenceStorageBackendKind,
    ) -> PlatformCapabilityReport {
        let report = self.planning_capability_report();
        if self.enabled {
            report.with_preference_storage_backend(backend)
        } else {
            report
        }
    }

    pub fn diagnostic_lines(&self) -> Vec<String> {
        self.diagnostic_lines_with_preference_storage_backend(
            PreferenceStorageBackendKind::Unavailable,
        )
    }

    pub fn diagnostic_lines_with_preference_storage_backend(
        &self,
        backend: PreferenceStorageBackendKind,
    ) -> Vec<String> {
        let mut lines = Vec::with_capacity(29);
        lines.push(format!("platform.enabled={}", self.enabled));
        lines.extend(
            self.planning_capability_report_with_preference_storage_backend(backend)
                .diagnostic_lines(),
        );
        lines
    }

    pub fn format_diagnostics(&self) -> String {
        self.diagnostic_lines().join("\n")
    }
}

impl Default for PlatformConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            target: PlatformTarget::current(),
            target_mode: RuntimeTargetMode::ClientRuntime,
            features: PlatformFeatureSelection::from_compiled_features(),
        }
    }
}
