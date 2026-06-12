use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_module_line_targets_state_child_mutates_pending_modes() {
    assert!(
        PARSER_LINE_MODULE_TARGETS_STATE.contains("*target_modes = modes"),
        "module targets state child should own pending target-mode mutation"
    );
}
