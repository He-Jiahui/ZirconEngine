mod row;
mod rows;

use super::projection::RuntimePluginFeatureValidationProjection;
use crate::plugin::PluginFeatureBundleManifest;

pub(super) fn validate_runtime_plugin_feature_modules(
    feature: &PluginFeatureBundleManifest,
    projection: &RuntimePluginFeatureValidationProjection<'_, '_>,
    diagnostics: &mut Vec<String>,
) {
    rows::validate_runtime_plugin_feature_module_rows(feature, projection, diagnostics);
}
