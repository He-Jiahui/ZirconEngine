use super::super::super::super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn set_enabled_by_default(feature: &mut PendingOptionalFeatureManifest, value: bool) {
    feature.enabled_by_default = Some(value);
}
