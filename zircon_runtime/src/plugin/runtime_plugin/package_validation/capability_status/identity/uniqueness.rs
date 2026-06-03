pub(super) fn validate_runtime_plugin_package_capability_status_uniqueness<'a>(
    capability: &'a str,
    seen_capabilities: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    if seen_capabilities.contains(&capability) {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{}` must be unique",
            capability
        ));
    } else {
        seen_capabilities.push(capability);
    }
}
