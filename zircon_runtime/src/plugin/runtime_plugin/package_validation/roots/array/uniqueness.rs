pub(super) fn validate_runtime_plugin_package_root_uniqueness<'a>(
    field_name: &str,
    root: &'a str,
    seen: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    if seen.contains(&root) {
        diagnostics.push(format!(
            "runtime plugin package manifest {field_name} root `{root}` must be unique"
        ));
    } else {
        seen.push(root);
    }
}
