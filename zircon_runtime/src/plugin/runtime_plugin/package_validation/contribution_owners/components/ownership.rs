pub(super) fn validate_runtime_plugin_package_component_owner(
    component_type_id: &str,
    component_plugin_id: &str,
    package_id: &str,
    diagnostics: &mut Vec<String>,
) {
    if component_plugin_id != package_id {
        diagnostics.push(format!(
            "runtime plugin package manifest component type `{component_type_id}` plugin_id `{component_plugin_id}` must match package id `{package_id}`"
        ));
    }
}
