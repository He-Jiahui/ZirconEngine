pub(super) fn validate_runtime_plugin_package_bevy_reference_uniqueness(
    capability: &str,
    reference: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    if is_duplicate {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{capability}` bevy reference `{reference}` must be unique"
        ));
    }
}
