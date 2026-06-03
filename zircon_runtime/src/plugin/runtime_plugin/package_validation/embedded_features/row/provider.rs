use crate::plugin::{PluginFeatureBundleManifest, PluginPackageManifest};

use super::super::super::embedded_feature_providers::validate_runtime_plugin_package_feature_provider;

pub(super) fn validate_runtime_plugin_package_embedded_feature_provider(
    field_name: &str,
    feature: &PluginFeatureBundleManifest,
    package_manifest: &PluginPackageManifest,
    seen_feature_providers: &mut Vec<(String, String)>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_feature_provider(
        field_name,
        feature,
        package_manifest,
        seen_feature_providers,
        diagnostics,
    );
}
