use super::super::super::super::types::{
    OptionalFeatureDependencySignature, PendingOptionalFeatureManifest,
};
use super::super::parent_feature;

pub(super) fn append_optional_feature_dependency(
    feature: &mut Option<PendingOptionalFeatureManifest>,
    dependency: OptionalFeatureDependencySignature,
) {
    let parent = parent_feature::required_parent_feature(
        feature,
        "optional feature dependency should have a parent feature",
    );
    parent.dependencies.push(dependency);
}
