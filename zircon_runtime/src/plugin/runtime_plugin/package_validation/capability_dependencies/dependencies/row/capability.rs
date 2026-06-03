use crate::plugin::PluginDependencyManifest;

use super::super::capability::validate_runtime_plugin_package_dependency_capability;

pub(super) fn validate_runtime_plugin_package_dependency_row_capability<'a>(
    dependency: &'a PluginDependencyManifest,
    diagnostics: &mut Vec<String>,
) -> Option<&'a str> {
    validate_runtime_plugin_package_dependency_capability(dependency, diagnostics)
}
