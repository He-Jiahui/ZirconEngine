use super::super::super::super::types::StaticOptionalFeatureManifest;

pub(super) fn push_static_optional_feature_manifest(
    features: &mut Vec<StaticOptionalFeatureManifest>,
    feature: StaticOptionalFeatureManifest,
) {
    features.push(feature);
}
