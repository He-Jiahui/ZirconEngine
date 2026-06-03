pub(super) fn validate_runtime_plugin_package_root_segments(
    field_name: &str,
    root: &str,
    diagnostics: &mut Vec<String>,
) {
    if root
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} root `{root}` must not contain empty, current, or parent path segments"
        ));
    }
}
