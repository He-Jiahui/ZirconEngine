use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_pending_feature_leaf_children_keep_finalize_ownership() {
    assert!(
        PARSER_PENDING_FEATURE_NORMALIZE.contains("feature.capabilities.sort_unstable()")
            && PARSER_PENDING_FEATURE_OUTPUT.contains("features.push(feature)")
            && PARSER_PENDING_FEATURE_STATIC_MANIFEST.contains("StaticOptionalFeatureManifest"),
        "pending feature normalize, output, and static-manifest children should keep leaf ownership"
    );
}
