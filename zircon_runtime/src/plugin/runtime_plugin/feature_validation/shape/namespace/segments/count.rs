pub(super) fn validate_runtime_plugin_feature_namespace_segment_count(
    field_name: &str,
    value: &str,
    segments: &[&str],
    diagnostics: &mut Vec<String>,
) -> bool {
    if segments.len() >= 2 {
        return true;
    }

    diagnostics.push(format!(
        "runtime plugin feature manifest {field_name} `{value}` must use at least two dot-separated namespace segments"
    ));
    false
}
