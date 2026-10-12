use crate::plugin::{PluginFeatureBundleManifest, PluginPackageManifest};

use super::super::super::embedded_feature_targets::validate_runtime_plugin_package_feature_target_coverage;

pub(super) fn validate_runtime_plugin_package_embedded_feature_target_coverage(
    field_name: &str,
    feature: &PluginFeatureBundleManifest,
    package_manifest: &PluginPackageManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_feature_target_coverage(
        field_name,
        feature,
        package_manifest,
        diagnostics,
    );
}
