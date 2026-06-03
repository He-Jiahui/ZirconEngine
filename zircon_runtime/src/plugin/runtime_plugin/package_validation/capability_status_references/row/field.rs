use super::super::field::validate_runtime_plugin_package_bevy_reference_field;

pub(super) fn validate_runtime_plugin_package_capability_status_bevy_reference_row_field(
    reference: &str,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_bevy_reference_field(reference, diagnostics);
}
