pub(super) fn validate_runtime_plugin_package_namespace_segment_count(
    field_name: &str,
    value: &str,
    diagnostics: &mut Vec<String>,
) -> bool {
    if value.contains('.') {
        return true;
    }

    diagnostics.push(format!(
        "runtime plugin package manifest {field_name} `{value}` must use at least two dot-separated namespace segments"
    ));
    false
}
