use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_transition_facade_stays_structural() {
    assert!(
        PARSER_STATE_TRANSITION.contains("mod dependency;")
            && PARSER_STATE_TRANSITION.contains("mod entry;")
            && PARSER_STATE_TRANSITION.contains("mod feature;")
            && PARSER_STATE_TRANSITION.contains("mod module;")
            && PARSER_STATE_TRANSITION.contains("mod pending_rows;"),
        "parser state transition parent must remain a structural child-module owner"
    );
    assert!(
        !PARSER_STATE_TRANSITION.contains("fn enter_section")
            && !PARSER_STATE_TRANSITION.contains("match section")
            && !PARSER_STATE_TRANSITION.contains("OptionalFeatureSection")
            && !PARSER_STATE_TRANSITION.contains("flush::"),
        "parser state transition parent must not own section-entry routing"
    );
    assert!(
        PARSER_STATE_TRANSITION.contains("use entry::enter_section"),
        "parser state transition parent should expose the entry child through a re-export"
    );
}
