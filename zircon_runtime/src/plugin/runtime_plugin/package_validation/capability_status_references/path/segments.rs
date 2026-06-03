pub(super) fn validate_runtime_plugin_package_bevy_reference_path_segments(
    capability: &str,
    reference: &str,
    diagnostics: &mut Vec<String>,
) {
    if reference
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{capability}` bevy reference `{reference}` must not contain empty, current, or parent path segments"
        ));
    }
}
