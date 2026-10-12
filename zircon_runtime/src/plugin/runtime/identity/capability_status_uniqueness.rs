pub(super) fn validate_runtime_plugin_package_capability_status_uniqueness(
    capability: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    if is_duplicate {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{}` must be unique",
            capability
        ));
    }
}
