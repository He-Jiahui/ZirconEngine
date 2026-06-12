use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_pending_dependency_entry_owner_composes_signature_append() {
    assert!(
        PARSER_PENDING_DEPENDENCY_ENTRY
            .contains("pub(in super::super::super) fn push_optional_feature_dependency")
            && PARSER_PENDING_DEPENDENCY_ENTRY
                .contains("super::signature::take_optional_feature_dependency")
            && PARSER_PENDING_DEPENDENCY_ENTRY
                .contains("super::append::append_optional_feature_dependency"),
        "pending dependency entry child should own signature-gated append composition"
    );
}
