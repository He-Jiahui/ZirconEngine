use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_module_line_kind_state_child_mutates_pending_kind() {
    assert!(
        PARSER_LINE_MODULE_KIND_STATE.contains("*kind = Some(value)"),
        "module kind state child should own pending module-kind mutation"
    );
}
