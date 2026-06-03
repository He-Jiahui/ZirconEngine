mod coverage;
mod module;
mod modules;

use crate::plugin::{PluginFeatureBundleManifest, PluginPackageManifest};

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_feature_target_coverage(
    field_name: &str,
    feature: &PluginFeatureBundleManifest,
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    modules::validate_runtime_plugin_package_feature_module_target_coverage(
        field_name,
        feature,
        package_manifest,
        diagnostics,
    );
}
