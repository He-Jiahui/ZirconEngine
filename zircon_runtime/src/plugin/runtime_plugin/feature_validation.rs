use crate::plugin::PluginFeatureBundleManifest;

mod capabilities;
mod dependencies;
mod identity;
mod modules;
mod provider;
mod shape;

use self::{
    capabilities::validate_runtime_plugin_feature_capabilities,
    dependencies::validate_runtime_plugin_feature_dependencies,
    identity::validate_runtime_plugin_feature_identity,
    modules::validate_runtime_plugin_feature_modules,
};
use super::package_validation::validate_runtime_plugin_default_packaging;

pub(super) fn validate_runtime_plugin_feature_manifest(
    feature: &PluginFeatureBundleManifest,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_identity(feature, diagnostics);
    validate_runtime_plugin_feature_capabilities(&feature.capabilities, diagnostics);
    validate_runtime_plugin_feature_dependencies(feature, diagnostics);
    validate_runtime_plugin_feature_modules(feature, diagnostics);
    validate_runtime_plugin_default_packaging(
        "feature manifest",
        &feature.default_packaging,
        diagnostics,
    );
}

pub(super) fn validate_runtime_plugin_feature_provider_package_id(
    provider_package_id: &str,
    diagnostics: &mut Vec<String>,
) {
    provider::validate_runtime_plugin_feature_provider_package_id(provider_package_id, diagnostics);
}
