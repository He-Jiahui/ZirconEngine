pub(super) fn validate_runtime_plugin_package_ui_component_id_uniqueness<'a>(
    ui_component_id: &'a str,
    seen_ui_component_ids: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    if seen_ui_component_ids.contains(&ui_component_id) {
        diagnostics.push(format!(
            "runtime plugin package manifest ui component `{ui_component_id}` must be unique"
        ));
    } else {
        seen_ui_component_ids.push(ui_component_id);
    }
}
