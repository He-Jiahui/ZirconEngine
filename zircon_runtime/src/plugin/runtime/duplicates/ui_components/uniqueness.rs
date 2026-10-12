pub(super) fn validate_runtime_plugin_package_ui_component_id_uniqueness(
    ui_component_id: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    if is_duplicate {
        diagnostics.push(format!(
            "runtime plugin package manifest ui component `{ui_component_id}` must be unique"
        ));
    }
}
