use super::super::super::super::types::{
    PendingOptionalFeatureManifest, StaticOptionalFeatureManifest,
};

pub(in super::super::super) fn push_optional_feature(
    features: &mut Vec<StaticOptionalFeatureManifest>,
    feature: &mut Option<PendingOptionalFeatureManifest>,
) {
    let Some(mut feature) = feature.take() else {
        return;
    };
    super::normalize::normalize_optional_feature(&mut feature);
    super::output::push_static_optional_feature_manifest(
        features,
        super::static_manifest::static_optional_feature_manifest(feature),
    );
}
