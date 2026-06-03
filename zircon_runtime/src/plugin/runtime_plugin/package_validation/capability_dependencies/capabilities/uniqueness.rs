pub(super) fn validate_runtime_plugin_package_capability_uniqueness<'a>(
    capability: &'a str,
    seen: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    if seen.contains(&capability) {
        diagnostics.push(format!(
            "runtime plugin package manifest capability `{capability}` must be unique"
        ));
    } else {
        seen.push(capability);
    }
}
