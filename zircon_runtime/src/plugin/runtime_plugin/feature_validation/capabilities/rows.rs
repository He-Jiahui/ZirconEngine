use super::super::projection::RuntimePluginFeatureValidationProjection;
use super::row::validate_runtime_plugin_feature_capability_row;

pub(super) fn validate_runtime_plugin_feature_capability_rows(
    capabilities: &[String],
    projection: &RuntimePluginFeatureValidationProjection<'_, '_>,
    diagnostics: &mut Vec<String>,
) {
    for (index, capability) in capabilities.iter().enumerate() {
        validate_runtime_plugin_feature_capability_row(
            capability,
            projection.capability_is_duplicate(index),
            diagnostics,
        );
    }
}
