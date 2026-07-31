mod provider;
mod target_coverage;

use super::super::projection::{EmbeddedFeatureKind, RuntimePluginPackageValidationProjection};
use crate::plugin::{PluginFeatureBundleManifest, PluginPackageManifest};

use self::{
    provider::validate_runtime_plugin_package_embedded_feature_provider,
    target_coverage::validate_runtime_plugin_package_embedded_feature_target_coverage,
};
use super::manifest::validate_runtime_plugin_package_embedded_feature_manifest;

pub(super) fn validate_runtime_plugin_package_embedded_feature_row(
    field_name: &str,
    feature: &PluginFeatureBundleManifest,
    package_manifest: &PluginPackageManifest,
    kind: EmbeddedFeatureKind,
    feature_index: usize,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_package_embedded_feature_manifest(
        feature,
        kind,
        feature_index,
        projection,
        diagnostics,
    );
    validate_runtime_plugin_package_embedded_feature_provider(
        field_name,
        feature,
        package_manifest,
        projection.embedded_feature_provider_is_duplicate(kind, feature_index),
        diagnostics,
    );
    validate_runtime_plugin_package_embedded_feature_target_coverage(
        field_name,
        feature,
        package_manifest,
        diagnostics,
    );
}
