use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_pending_dependency_leaf_children_keep_finalize_ownership() {
    assert!(
        PARSER_PENDING_DEPENDENCY_SIGNATURE.contains("let plugin_id = plugin_id.take()?")
            && PARSER_PENDING_DEPENDENCY_APPEND.contains("parent.dependencies.push(dependency)"),
        "pending dependency signature and append children should keep leaf ownership"
    );
}
