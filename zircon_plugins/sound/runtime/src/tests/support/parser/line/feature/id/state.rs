use super::super::super::super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn set_feature_id(feature: &mut PendingOptionalFeatureManifest, value: String) {
    feature.id = Some(value);
}
