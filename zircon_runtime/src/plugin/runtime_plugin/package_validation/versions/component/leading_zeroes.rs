pub(super) fn validate_runtime_plugin_package_semver_component_leading_zeroes(
    field_name: &str,
    value: &str,
    component_name: &str,
    segment: &str,
    diagnostics: &mut Vec<String>,
) -> bool {
    if segment != "0" && segment.starts_with('0') {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{value}` {component_name} component `{segment}` must not use leading zeroes"
        ));
        return false;
    }
    true
}
