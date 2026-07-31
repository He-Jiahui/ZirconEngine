mod namespace;
mod ownership;
mod uniqueness;

pub(super) fn validate_runtime_plugin_package_capability_status_identity(
    capability: &str,
    is_owned: bool,
    is_duplicate: bool,
    diagnostics: &mut Vec<String>,
) {
    namespace::validate_runtime_plugin_package_capability_status_namespace(capability, diagnostics);
    ownership::validate_runtime_plugin_package_capability_status_ownership(
        capability,
        is_owned,
        diagnostics,
    );
    uniqueness::validate_runtime_plugin_package_capability_status_uniqueness(
        capability,
        is_duplicate,
        diagnostics,
    );
}
