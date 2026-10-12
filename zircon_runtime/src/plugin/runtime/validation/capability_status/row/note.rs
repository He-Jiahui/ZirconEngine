use crate::plugin::CapabilityStatusManifest;

use super::super::note::validate_runtime_plugin_package_capability_status_note;

pub(super) fn validate_runtime_plugin_package_capability_status_row_note(
    status: &CapabilityStatusManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_capability_status_note(status, diagnostics);
}
