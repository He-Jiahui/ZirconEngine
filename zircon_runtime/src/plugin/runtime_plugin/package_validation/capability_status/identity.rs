mod namespace;
mod ownership;
mod uniqueness;

pub(super) fn validate_runtime_plugin_package_capability_status_identity<'a>(
    capability: &'a str,
    owned_capabilities: &[&str],
    seen_capabilities: &mut Vec<&'a str>,
    diagnostics: &mut Vec<String>,
) {
    namespace::validate_runtime_plugin_package_capability_status_namespace(capability, diagnostics);
    ownership::validate_runtime_plugin_package_capability_status_ownership(
        capability,
        owned_capabilities,
        diagnostics,
    );
    uniqueness::validate_runtime_plugin_package_capability_status_uniqueness(
        capability,
        seen_capabilities,
        diagnostics,
    );
}
