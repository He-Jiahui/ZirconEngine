pub(super) fn validate_runtime_plugin_package_root_field(
    field_name: &str,
    root: &str,
    diagnostics: &mut Vec<String>,
) -> bool {
    if root.trim().is_empty() || root.trim() != root {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} root `{root}` must be non-empty and trimmed"
        ));
        return false;
    }
    true
}
