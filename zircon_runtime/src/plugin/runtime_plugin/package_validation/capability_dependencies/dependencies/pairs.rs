/// 唯一性属于提供包与能力的组合；同一个提供包可以通过不同声明提供不同能力。
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
