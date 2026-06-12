use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_pending_dependency_facade_stays_structural() {
    assert!(
        PARSER_PENDING_DEPENDENCY.contains("mod append;")
            && PARSER_PENDING_DEPENDENCY.contains("mod entry;")
            && PARSER_PENDING_DEPENDENCY.contains("mod signature;"),
        "pending dependency parent must remain a structural child-module owner"
    );
    assert!(
        !PARSER_PENDING_DEPENDENCY.contains("fn push_optional_feature_dependency")
            && !PARSER_PENDING_DEPENDENCY.contains("take_optional_feature_dependency")
            && !PARSER_PENDING_DEPENDENCY.contains("append_optional_feature_dependency")
            && !PARSER_PENDING_DEPENDENCY.contains("PendingOptionalFeatureManifest"),
        "pending dependency parent must not own finalizer composition or dependency state signature"
    );
    assert!(
        PARSER_PENDING_DEPENDENCY.contains("use entry::push_optional_feature_dependency"),
        "pending dependency parent should expose the entry child through a re-export"
    );
}
