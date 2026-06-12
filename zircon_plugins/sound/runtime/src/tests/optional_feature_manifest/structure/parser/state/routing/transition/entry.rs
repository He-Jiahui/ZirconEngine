use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_transition_entry_owner_routes_section_entry() {
    assert!(
        PARSER_STATE_TRANSITION_ENTRY.contains("pub(in super::super) fn enter_section")
            && PARSER_STATE_TRANSITION_ENTRY.contains("match section")
            && PARSER_STATE_TRANSITION_ENTRY.contains("super::feature::start_optional_feature")
            && PARSER_STATE_TRANSITION_ENTRY.contains("super::dependency::start_dependency")
            && PARSER_STATE_TRANSITION_ENTRY.contains("super::module::start_module")
            && PARSER_STATE_TRANSITION_ENTRY.contains("flush::close_optional_feature_scope"),
        "parser state transition entry child should own section-entry routing"
    );
}
