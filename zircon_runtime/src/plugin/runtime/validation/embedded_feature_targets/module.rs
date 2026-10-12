use crate::plugin::{PluginModuleManifest, PluginPackageManifest};

use super::coverage::validate_runtime_plugin_package_feature_target_coverage;

pub(super) fn validate_runtime_plugin_package_feature_module_target_coverage(
    field_name: &str,
    feature_id: &str,
    module: &PluginModuleManifest,
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    for target_mode in module.target_modes.iter().copied() {
        validate_runtime_plugin_package_feature_target_coverage(
            field_name,
            feature_id,
            &module.name,
            package_manifest,
            target_mode,
            diagnostics,
        );
    }
}
