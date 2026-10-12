pub(super) fn validate_runtime_plugin_package_semver_segment_count(
    field_name: &str,
    value: &str,
    segment_count: usize,
    diagnostics: &mut Vec<String>,
) -> bool {
    if segment_count == 3 {
        return true;
    }
    diagnostics.push(format!(
        "runtime plugin package manifest {field_name} `{value}` must use MAJOR.MINOR.PATCH form"
    ));
    false
}
