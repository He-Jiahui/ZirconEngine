use super::super::super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn set_feature_capabilities(
    feature: &mut PendingOptionalFeatureManifest,
    values: Vec<String>,
) {
    feature.capabilities = values;
}
