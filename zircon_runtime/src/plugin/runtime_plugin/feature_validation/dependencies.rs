mod owner;
mod pairs;
mod presence;
mod primary_count;
mod row;
mod rows;

use super::projection::RuntimePluginFeatureValidationProjection;
use crate::plugin::PluginFeatureBundleManifest;

pub(super) fn validate_runtime_plugin_feature_dependencies(
    feature: &PluginFeatureBundleManifest,
    projection: &RuntimePluginFeatureValidationProjection<'_, '_>,
    diagnostics: &mut Vec<String>,
) {
    presence::validate_runtime_plugin_feature_dependency_presence(feature, diagnostics);
    rows::validate_runtime_plugin_feature_dependency_rows(feature, projection, diagnostics);
}
