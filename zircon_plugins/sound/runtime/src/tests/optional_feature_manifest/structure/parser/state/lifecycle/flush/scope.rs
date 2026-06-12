use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_flush_scope_child_keeps_ordered_scope_closure() {
    assert!(
        PARSER_STATE_FLUSH_SCOPE.contains("dependency::flush_pending_dependency")
            && PARSER_STATE_FLUSH_SCOPE.contains("module::flush_pending_module")
            && PARSER_STATE_FLUSH_SCOPE.contains("feature::flush_pending_feature")
            && PARSER_STATE_FLUSH_SCOPE.contains("state.section = OptionalFeatureSection::None"),
        "parser state flush scope child should own ordered scope closure"
    );
}
