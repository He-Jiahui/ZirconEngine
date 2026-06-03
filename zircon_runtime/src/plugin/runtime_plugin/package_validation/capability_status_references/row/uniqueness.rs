use super::super::uniqueness::validate_runtime_plugin_package_bevy_reference_uniqueness;

pub(super) fn validate_runtime_plugin_package_capability_status_bevy_reference_row_uniqueness<
    'a,
>(
    capability: &str,
    reference: &'a str,
    seen: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_bevy_reference_uniqueness(
        capability,
        reference,
        seen,
        diagnostics,
    );
}
