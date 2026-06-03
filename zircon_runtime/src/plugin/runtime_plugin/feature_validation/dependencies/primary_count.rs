pub(super) fn validate_runtime_plugin_feature_primary_dependency_count(
    primary_count: usize,
    diagnostics: &mut Vec<String>,
) {
    if primary_count != 1 {
        diagnostics.push(format!(
            "runtime plugin feature manifest dependencies must declare exactly one primary dependency, found {primary_count}"
        ));
    }
}
