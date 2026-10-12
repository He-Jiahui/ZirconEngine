use super::super::super::validate_runtime_plugin_package_namespace;

pub(super) fn validate_runtime_plugin_package_capability_status_namespace(
    capability: &str,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_namespace(
        "capability status capability",
        capability,
        diagnostics,
    );
}
