use crate::plugin::PluginPackageManifest;

use super::super::projection::RuntimePluginPackageValidationProjection;
use super::row::validate_runtime_plugin_package_module_row;

pub(super) fn validate_runtime_plugin_package_module_rows(
    package_manifest: &PluginPackageManifest,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    for (module_index, module) in package_manifest.modules.iter().enumerate() {
        validate_runtime_plugin_package_module_row(
            &package_manifest.id,
            package_manifest.supported_targets.as_slice(),
            module,
            module_index,
            projection,
            diagnostics,
        );
    }
}
