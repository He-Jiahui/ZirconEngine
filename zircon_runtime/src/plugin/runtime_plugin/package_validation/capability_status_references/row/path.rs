use super::super::path::validate_runtime_plugin_package_bevy_reference_path;

pub(super) fn validate_runtime_plugin_package_capability_status_bevy_reference_row_path(
    capability: &str,
    reference: &str,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_bevy_reference_path(capability, reference, diagnostics);
}
