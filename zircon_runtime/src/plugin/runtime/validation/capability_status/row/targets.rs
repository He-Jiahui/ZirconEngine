use crate::plugin::{CapabilityStatusManifest, PluginPackageManifest};

use super::super::super::capability_status_targets::validate_runtime_plugin_package_capability_status_targets;

pub(super) fn validate_runtime_plugin_package_capability_status_row_targets(
    package_manifest: &PluginPackageManifest,
    status: &CapabilityStatusManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_capability_status_targets(
        package_manifest,
        &status.capability,
        &status.target_modes,
        diagnostics,
    );
}
