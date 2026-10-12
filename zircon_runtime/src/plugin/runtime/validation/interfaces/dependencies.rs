use crate::plugin::PluginPackageManifest;

use super::super::{
    projection::RuntimePluginPackageValidationProjection, validate_runtime_plugin_package_namespace,
};

pub(super) fn validate_runtime_plugin_package_dependency_interfaces(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (dependency_index, dependency) in package_manifest.dependencies.iter().enumerate() {
        for (interface_index, interface_id) in dependency.interfaces.iter().enumerate() {
            validate_runtime_plugin_package_namespace(
                "dependency interface id",
                interface_id,
                diagnostics,
            );
            if projection.dependency_interface_is_duplicate(dependency_index, interface_index) {
                diagnostics.push(format!(
                    "runtime plugin package manifest dependency `{}` interface `{}` must be unique",
                    dependency.id, interface_id
                ));
            }
        }
    }
}
