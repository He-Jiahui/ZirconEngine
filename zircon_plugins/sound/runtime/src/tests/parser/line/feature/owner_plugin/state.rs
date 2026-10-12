use super::super::super::super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn set_feature_owner_plugin_id(
    feature: &mut PendingOptionalFeatureManifest,
    value: String,
) {
    feature.owner_plugin_id = Some(value);
}
