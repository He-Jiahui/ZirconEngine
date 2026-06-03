use crate::plugin::CapabilityStatusManifest;

use super::super::super::capability_status_references::validate_runtime_plugin_package_capability_status_bevy_references;

pub(super) fn validate_runtime_plugin_package_capability_status_row_bevy_references(
    status: &CapabilityStatusManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_capability_status_bevy_references(
        &status.capability,
        &status.bevy_references,
        diagnostics,
    );
}
