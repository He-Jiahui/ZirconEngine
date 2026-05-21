use serde::{Deserialize, Serialize};

use crate::RuntimeTargetMode;

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
    pub fn capability_report(&self) -> PlatformCapabilityReport {
        PlatformCapabilityMatrix::new(self.features).report(self.target, self.target_mode)
    }

    pub fn diagnostic_lines(&self) -> Vec<String> {
        let mut lines = Vec::with_capacity(28);
        lines.push(format!("platform.enabled={}", self.enabled));
        lines.extend(self.capability_report().diagnostic_lines());
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
