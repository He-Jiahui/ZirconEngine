use crate::plugin::PluginFeatureBundleManifest;

mod capabilities;
mod dependencies;
mod identity;
mod modules;
mod projection;
mod provider;
mod shape;

#[cfg(test)]
mod tests;

use self::projection::RuntimePluginFeatureValidationProjection;
#[cfg(test)]
pub(super) use self::projection::{
    begin_feature_projection_build_observation, observed_embedded_feature_projection_views,
    observed_standalone_feature_projection_builds,
};
use self::{
    capabilities::validate_runtime_plugin_feature_capabilities,
    dependencies::validate_runtime_plugin_feature_dependencies,
    identity::validate_runtime_plugin_feature_identity,
    modules::validate_runtime_plugin_feature_modules,
};
use super::package_validation::validate_runtime_plugin_default_packaging;
use super::package_validation::{EmbeddedFeatureKind, RuntimePluginPackageValidationProjection};

pub(super) fn validate_runtime_plugin_feature_manifest(
    feature: &PluginFeatureBundleManifest,
    diagnostics: &mut Vec<String>,
) {
    let projection = RuntimePluginFeatureValidationProjection::standalone(feature);
    validate_runtime_plugin_feature_manifest_with_projection(feature, &projection, diagnostics);
}

pub(super) fn validate_runtime_plugin_embedded_feature_manifest(
    feature: &PluginFeatureBundleManifest,
    kind: EmbeddedFeatureKind,
    feature_index: usize,
    package_projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    let projection =
        RuntimePluginFeatureValidationProjection::embedded(package_projection, kind, feature_index);
    validate_runtime_plugin_feature_manifest_with_projection(feature, &projection, diagnostics);
}

fn validate_runtime_plugin_feature_manifest_with_projection(
    feature: &PluginFeatureBundleManifest,
    projection: &RuntimePluginFeatureValidationProjection<'_, '_>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_feature_identity(feature, diagnostics);
    if let Some(provider_package_id) = feature.provider_package_id.as_deref() {
        provider::validate_runtime_plugin_feature_provider_package_id(
            provider_package_id,
            diagnostics,
        );
    }
    validate_runtime_plugin_feature_capabilities(&feature.capabilities, projection, diagnostics);
    validate_runtime_plugin_feature_dependencies(feature, projection, diagnostics);
    validate_runtime_plugin_feature_modules(feature, projection, diagnostics);
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
