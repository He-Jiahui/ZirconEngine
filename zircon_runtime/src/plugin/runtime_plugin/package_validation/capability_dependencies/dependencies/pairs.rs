pub(super) fn validate_runtime_plugin_package_dependency_pair(
    dependency_id: &str,
    capability: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    if is_duplicate {
        diagnostics.push(format!(
            "runtime plugin package manifest dependency `{dependency_id}` capability `{capability}` must be unique",
        ));
    }
}
