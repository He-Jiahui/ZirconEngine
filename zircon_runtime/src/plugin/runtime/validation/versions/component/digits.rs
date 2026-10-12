pub(super) fn validate_runtime_plugin_package_semver_component_digits(
    field_name: &str,
    value: &str,
    component_name: &str,
    segment: &str,
    diagnostics: &mut Vec<String>,
) -> bool {
    if segment.is_empty() || !segment.bytes().all(|byte| byte.is_ascii_digit()) {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} `{value}` {component_name} component `{segment}` must contain ASCII digits"
        ));
        return false;
    }
    true
}
