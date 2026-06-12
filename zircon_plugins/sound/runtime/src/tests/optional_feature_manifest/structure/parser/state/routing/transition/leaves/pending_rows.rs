use super::super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_transition_pending_row_leaves_keep_handoffs() {
    assert!(
        PARSER_STATE_TRANSITION_DEPENDENCY
            .contains("pending_rows::flush_dependency_and_module_rows")
            && PARSER_STATE_TRANSITION_MODULE
                .contains("pending_rows::flush_dependency_and_module_rows")
            && PARSER_STATE_TRANSITION_PENDING_ROWS.contains("flush::flush_pending_dependency")
            && PARSER_STATE_TRANSITION_PENDING_ROWS.contains("flush::flush_pending_module"),
        "parser state transition pending-row leaves should own dependency/module handoffs"
    );
}
