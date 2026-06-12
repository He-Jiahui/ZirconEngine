use crate::plugin::PluginPackageManifest;

use super::super::validate_runtime_plugin_package_namespace;

pub(super) fn validate_runtime_plugin_package_provided_interfaces(
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    let mut seen = Vec::new();
    for interface in &package_manifest.provides_interfaces {
        validate_runtime_plugin_package_namespace(
            "provided interface id",
            interface.id.as_str(),
            diagnostics,
        );
        if seen.contains(&interface.id.as_str()) {
            diagnostics.push(format!(
                "runtime plugin package manifest provided interface `{}` must be unique",
                interface.id
            ));
        } else {
            seen.push(interface.id.as_str());
        }
    }
}
