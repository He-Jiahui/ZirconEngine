pub(super) fn validate_runtime_plugin_package_component_type_uniqueness<'a>(
    component_type_id: &'a str,
    seen_component_type_ids: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    if seen_component_type_ids.contains(&component_type_id) {
        diagnostics.push(format!(
            "runtime plugin package manifest component type `{component_type_id}` must be unique"
        ));
    } else {
        seen_component_type_ids.push(component_type_id);
    }
}
