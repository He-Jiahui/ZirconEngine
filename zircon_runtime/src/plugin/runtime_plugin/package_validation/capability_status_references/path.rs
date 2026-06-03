mod segments;

pub(super) fn validate_runtime_plugin_package_bevy_reference_path(
    capability: &str,
    reference: &str,
    diagnostics: &mut Vec<String>,
) {
    if !reference.starts_with("dev/bevy/") {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{capability}` bevy reference `{reference}` must stay under `dev/bevy`"
        ));
    }
    if reference.contains('\\') || reference.contains(':') {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{capability}` bevy reference `{reference}` must be a repository-relative forward-slash path"
        ));
    }
    segments::validate_runtime_plugin_package_bevy_reference_path_segments(
        capability,
        reference,
        diagnostics,
    );
}
