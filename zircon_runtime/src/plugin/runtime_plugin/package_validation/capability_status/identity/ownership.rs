pub(super) fn validate_runtime_plugin_package_capability_status_ownership(
    capability: &str,
    owned_capabilities: &[&str],
    diagnostics: &mut Vec<String>,
) {
    if !owned_capabilities.contains(&capability) {
        diagnostics.push(format!(
            "runtime plugin package manifest capability status `{}` must reference a package or optional feature capability declared by the same package",
            capability
        ));
    }
}
