pub(super) fn validate_runtime_plugin_package_component_type_uniqueness(
    component_type_id: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    if is_duplicate {
        diagnostics.push(format!(
            "runtime plugin package manifest component type `{component_type_id}` must be unique"
        ));
    }
}
