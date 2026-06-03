pub(super) fn validate_runtime_plugin_package_event_catalog_namespace_uniqueness<'a>(
    event_catalog_namespace: &'a str,
    seen_event_catalog_namespaces: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    if seen_event_catalog_namespaces.contains(&event_catalog_namespace) {
        diagnostics.push(format!(
            "runtime plugin package manifest event catalog namespace `{event_catalog_namespace}` must be unique"
        ));
    } else {
        seen_event_catalog_namespaces.push(event_catalog_namespace);
    }
}
