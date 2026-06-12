use super::super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_transition_feature_leaf_starts_pending_feature() {
    assert!(
        PARSER_STATE_TRANSITION_FEATURE
            .contains("state.current_feature = Some(Default::default())"),
        "parser state transition feature leaf should own pending-feature start"
    );
}
