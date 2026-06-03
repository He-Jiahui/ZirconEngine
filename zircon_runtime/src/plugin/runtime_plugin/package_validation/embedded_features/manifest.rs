use crate::plugin::PluginFeatureBundleManifest;

use super::super::super::feature_validation::validate_runtime_plugin_feature_manifest;

pub(super) fn validate_runtime_plugin_package_embedded_feature_manifest(
    feature: &PluginFeatureBundleManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_manifest(feature, diagnostics);
}
