// 功能提交前规范化集合顺序，使静态与运行时签名按内容比较。
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
