use crate::plugin::{PluginFeatureBundleManifest, PluginPackageManifest};

use super::module::validate_runtime_plugin_package_feature_module_target_coverage as validate_runtime_plugin_package_feature_module_row_target_coverage;

pub(super) fn validate_runtime_plugin_package_feature_module_target_coverage(
    field_name: &str,
    feature: &PluginFeatureBundleManifest,
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    for module in &feature.modules {
        validate_runtime_plugin_package_feature_module_row_target_coverage(
            field_name,
            &feature.id,
            module,
            package_manifest,
            diagnostics,
        );
    }
}
