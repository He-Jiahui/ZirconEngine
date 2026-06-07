use super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn required_parent_feature<'a>(
    feature: &'a mut Option<PendingOptionalFeatureManifest>,
    message: &'static str,
) -> &'a mut PendingOptionalFeatureManifest {
    feature.as_mut().expect(message)
}
