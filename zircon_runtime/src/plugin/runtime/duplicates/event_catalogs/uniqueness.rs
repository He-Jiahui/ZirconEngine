pub(super) fn validate_runtime_plugin_package_event_catalog_namespace_uniqueness(
    event_catalog_namespace: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    if is_duplicate {
        diagnostics.push(format!(
            "runtime plugin package manifest event catalog namespace `{event_catalog_namespace}` must be unique"
        ));
    }
}
