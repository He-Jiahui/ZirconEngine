use crate::plugin::CapabilityStatusManifest;

use super::super::validate_runtime_plugin_package_field;

pub(super) fn validate_runtime_plugin_package_capability_status_note(
    status: &CapabilityStatusManifest,
    diagnostics: &mut Vec<String>,
) {
    if let Some(note) = status.note.as_deref() {
        validate_runtime_plugin_package_field("capability status note", note, diagnostics);
    }
}
