use super::super::super::super::types::PendingOptionalFeatureManifest;

pub(super) fn normalize_optional_feature(feature: &mut PendingOptionalFeatureManifest) {
    feature.capabilities.sort_unstable();
    feature.dependencies.sort_unstable();
    feature
        .modules
        .sort_unstable_by_key(|module| module.0.clone());
}
