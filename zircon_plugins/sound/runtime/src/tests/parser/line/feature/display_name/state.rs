use super::super::super::super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn set_feature_display_name(
    feature: &mut PendingOptionalFeatureManifest,
    value: String,
) {
    feature.display_name = Some(value);
}
