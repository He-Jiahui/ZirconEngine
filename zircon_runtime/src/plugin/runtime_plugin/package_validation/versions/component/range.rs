pub(super) fn validate_runtime_plugin_package_semver_component_range(
    field_name: &str,
    value: &str,
    component_name: &str,
    segment: &str,
    diagnostics: &mut Vec<String>,
) {
    if segment.parse::<u32>().is_err() {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{value}` {component_name} component `{segment}` must fit in u32"
        ));
    }
}
