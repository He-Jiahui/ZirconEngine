use crate::plugin::PluginDependencyManifest;

use super::super::super::super::validate_runtime_plugin_package_token;

pub(super) fn validate_runtime_plugin_package_dependency_provider(
    dependency: &PluginDependencyManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_token("dependency id", &dependency.id, diagnostics);
}
