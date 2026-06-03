use super::super::super::super::validate_runtime_plugin_package_namespace;

pub(super) fn validate_runtime_plugin_package_dependency_capability_namespace(
    capability: &str,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_namespace("dependency capability", capability, diagnostics);
}
