use super::super::projection::{EmbeddedFeatureKind, RuntimePluginPackageValidationProjection};
use crate::plugin::PluginFeatureBundleManifest;

use super::super::super::feature_validation::validate_runtime_plugin_embedded_feature_manifest;

pub(super) fn validate_runtime_plugin_package_embedded_feature_manifest(
    feature: &PluginFeatureBundleManifest,
    kind: EmbeddedFeatureKind,
    feature_index: usize,
    projection: &RuntimePluginPackageValidationProjection<'_>,
    diagnostics: &mut Vec<String>,
) {
    validate_runtime_plugin_embedded_feature_manifest(
        feature,
        kind,
        feature_index,
        projection,
        diagnostics,
    );
}
