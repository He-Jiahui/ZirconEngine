mod provider_id;
mod uniqueness;

use crate::plugin::{PluginFeatureBundleManifest, PluginPackageManifest};

use self::{
    provider_id::runtime_plugin_package_feature_provider_package_id,
    uniqueness::validate_runtime_plugin_package_feature_provider_uniqueness,
};

pub(in crate::plugin::runtime_plugin) fn validate_runtime_plugin_package_feature_provider(
    field_name: &str,
    feature: &PluginFeatureBundleManifest,
    package_manifest: &PluginPackageManifest,
    seen_feature_providers: &mut Vec<(String, String)>,
    diagnostics: &mut Vec<String>,
) {
    let provider_package_id =
        runtime_plugin_package_feature_provider_package_id(package_manifest, feature);
    validate_runtime_plugin_package_feature_provider_uniqueness(
        field_name,
        &feature.id,
        provider_package_id,
        seen_feature_providers,
        diagnostics,
    );
}
