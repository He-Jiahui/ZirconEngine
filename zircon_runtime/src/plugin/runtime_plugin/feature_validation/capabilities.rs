mod presence;
mod row;
mod rows;
mod uniqueness;

use super::projection::RuntimePluginFeatureValidationProjection;

pub(super) fn validate_runtime_plugin_feature_capabilities(
    capabilities: &[String],
    projection: &RuntimePluginFeatureValidationProjection<'_, '_>,
    diagnostics: &mut Vec<String>,
) {
    presence::validate_runtime_plugin_feature_capability_presence(capabilities, diagnostics);
    rows::validate_runtime_plugin_feature_capability_rows(capabilities, projection, diagnostics);
}
