use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_flush_facade_stays_structural() {
    assert!(
        PARSER_STATE_FLUSH.contains("mod dependency;")
            && PARSER_STATE_FLUSH.contains("mod feature;")
            && PARSER_STATE_FLUSH.contains("mod module;")
            && PARSER_STATE_FLUSH.contains("mod scope;"),
        "parser state flush parent must remain a structural child-module owner"
    );
    assert!(
        !PARSER_STATE_FLUSH.contains("fn flush_pending_dependency")
            && !PARSER_STATE_FLUSH.contains("fn flush_pending_feature")
            && !PARSER_STATE_FLUSH.contains("fn flush_pending_module")
            && !PARSER_STATE_FLUSH.contains("fn close_optional_feature_scope"),
        "parser state flush parent must not own row or scope flush bodies"
    );
    assert!(
        PARSER_STATE_FLUSH.contains("use self::dependency::flush_pending_dependency")
            && PARSER_STATE_FLUSH.contains("use self::module::flush_pending_module")
            && PARSER_STATE_FLUSH.contains("use self::scope::close_optional_feature_scope"),
        "parser state flush parent should expose child-owned flush helpers through re-exports"
    );
}
