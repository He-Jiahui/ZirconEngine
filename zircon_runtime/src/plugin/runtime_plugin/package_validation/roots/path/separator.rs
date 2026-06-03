pub(super) fn validate_runtime_plugin_package_root_separator(
    field_name: &str,
    root: &str,
    diagnostics: &mut Vec<String>,
) {
    if root.contains('\\') {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} root `{root}` must use forward slashes"
        ));
    }
}
