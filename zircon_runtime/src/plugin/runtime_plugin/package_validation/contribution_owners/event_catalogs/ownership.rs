pub(super) fn validate_runtime_plugin_package_event_catalog_owner(
    event_catalog_namespace: &str,
    package_id: &str,
    expected_prefix: &str,
    diagnostics: &mut Vec<String>,
) {
    if !event_catalog_namespace.starts_with(expected_prefix) {
        diagnostics.push(format!(
            "runtime plugin package manifest event catalog namespace `{event_catalog_namespace}` must be prefixed by package id `{package_id}`"
        ));
    }
}
