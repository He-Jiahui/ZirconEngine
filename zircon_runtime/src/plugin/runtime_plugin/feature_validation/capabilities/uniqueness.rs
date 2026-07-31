pub(super) fn validate_runtime_plugin_feature_capability_uniqueness(
    capability: &str,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    if is_duplicate {
        diagnostics.push(format!(
            "runtime plugin feature manifest capability `{capability}` must be unique"
        ));
    }
}
