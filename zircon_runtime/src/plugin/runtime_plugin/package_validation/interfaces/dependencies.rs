use crate::plugin::PluginPackageManifest;

use super::super::validate_runtime_plugin_package_namespace;

pub(super) fn validate_runtime_plugin_package_dependency_interfaces(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    for dependency in &package_manifest.dependencies {
        let mut seen = Vec::new();
        for interface_id in &dependency.interfaces {
            validate_runtime_plugin_package_namespace(
                "dependency interface id",
                interface_id,
                diagnostics,
            );
            if seen.contains(&interface_id.as_str()) {
                diagnostics.push(format!(
                    "runtime plugin package manifest dependency `{}` interface `{}` must be unique",
                    dependency.id, interface_id
                ));
            } else {
                seen.push(interface_id.as_str());
            }
        }
    }
}
