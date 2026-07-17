mod provider;
mod target_coverage;

use crate::plugin::{PluginFeatureBundleManifest, PluginPackageManifest};

use self::{
    provider::validate_runtime_plugin_package_embedded_feature_provider,
    target_coverage::validate_runtime_plugin_package_embedded_feature_target_coverage,
};
use super::manifest::validate_runtime_plugin_package_embedded_feature_manifest;

pub(super) fn validate_runtime_plugin_package_embedded_feature_row<'a>(
    field_name: &str,
    feature: &'a PluginFeatureBundleManifest,
    package_manifest: &'a PluginPackageManifest,
    seen_feature_providers: &mut Vec<(&'a str, &'a str)>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_embedded_feature_manifest(feature, diagnostics);
    validate_runtime_plugin_package_embedded_feature_provider(
        field_name,
        feature,
        package_manifest,
        seen_feature_providers,
        diagnostics,
    );
    validate_runtime_plugin_package_embedded_feature_target_coverage(
        field_name,
        feature,
        package_manifest,
        diagnostics,
    );
}
