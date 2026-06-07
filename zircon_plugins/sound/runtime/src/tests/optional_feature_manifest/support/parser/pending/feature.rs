mod normalize;
mod output;
mod static_manifest;

use super::super::super::types::{PendingOptionalFeatureManifest, StaticOptionalFeatureManifest};

pub(in super::super) fn push_optional_feature(
    features: &mut Vec<StaticOptionalFeatureManifest>,
    feature: &mut Option<PendingOptionalFeatureManifest>,
) {
    let Some(mut feature) = feature.take() else {
        return;
    };
    normalize::normalize_optional_feature(&mut feature);
    output::push_static_optional_feature_manifest(
        features,
        static_manifest::static_optional_feature_manifest(feature),
    );
}
