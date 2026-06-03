pub(super) fn validate_runtime_plugin_package_ui_component_owner(
    ui_component_id: &str,
    ui_component_plugin_id: &str,
    package_id: &str,
    diagnostics: &mut Vec<String>,
) {
    if ui_component_plugin_id != package_id {
        diagnostics.push(format!(
            "runtime plugin package manifest ui component `{ui_component_id}` plugin_id `{ui_component_plugin_id}` must match package id `{package_id}`"
        ));
    }
}
