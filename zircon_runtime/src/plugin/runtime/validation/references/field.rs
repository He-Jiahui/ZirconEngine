use super::super::validate_runtime_plugin_package_field;

pub(super) fn validate_runtime_plugin_package_bevy_reference_field(
    reference: &str,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_field(
        "capability status bevy_reference",
        reference,
        diagnostics,
    );
}
