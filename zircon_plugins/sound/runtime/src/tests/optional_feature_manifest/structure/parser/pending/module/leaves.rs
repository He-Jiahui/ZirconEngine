use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_pending_module_leaf_children_keep_finalize_ownership() {
    assert!(
        PARSER_PENDING_MODULE_SIGNATURE.contains("let name = name.take()?")
            && PARSER_PENDING_MODULE_APPEND.contains("parent.modules.push(module)"),
        "pending module signature and append children should keep leaf ownership"
    );
}
