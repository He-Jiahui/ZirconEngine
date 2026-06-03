pub(super) fn validate_runtime_plugin_package_root_relative(
    field_name: &str,
    root: &str,
    diagnostics: &mut Vec<String>,
) {
    if root.starts_with('/') || root.starts_with('\\') {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} root `{root}` must be relative"
        ));
    }
}
