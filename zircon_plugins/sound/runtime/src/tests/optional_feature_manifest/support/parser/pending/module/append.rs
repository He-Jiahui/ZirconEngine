use super::super::super::super::types::{
    OptionalFeatureModuleSignature, PendingOptionalFeatureManifest,
};
use super::super::parent_feature;

pub(super) fn append_optional_feature_module(
    feature: &mut Option<PendingOptionalFeatureManifest>,
    module: OptionalFeatureModuleSignature,
) {
    let parent = parent_feature::required_parent_feature(
        feature,
        "optional feature module should have a parent feature",
    );
    parent.modules.push(module);
}
