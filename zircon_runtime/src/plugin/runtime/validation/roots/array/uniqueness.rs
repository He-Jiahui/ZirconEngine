pub(super) fn validate_runtime_plugin_package_root_uniqueness(
    field_name: &str,
    root: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    if is_duplicate {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} root `{root}` must be unique"
        ));
    }
}
