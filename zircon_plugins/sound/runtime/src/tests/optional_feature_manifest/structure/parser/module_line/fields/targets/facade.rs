use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_module_line_targets_facade_stays_structural() {
    assert!(
        PARSER_LINE_MODULE_TARGETS.contains("mod field;")
            && PARSER_LINE_MODULE_TARGETS.contains("mod modes;")
            && PARSER_LINE_MODULE_TARGETS.contains("mod state;"),
        "module targets parent must remain split into field, modes, and state children"
    );
    assert!(
        PARSER_LINE_MODULE_TARGETS.contains("field::module_target_modes_value")
            && PARSER_LINE_MODULE_TARGETS.contains("modes::module_target_modes_from_plugin_toml")
            && PARSER_LINE_MODULE_TARGETS.contains("state::set_module_target_modes"),
        "module targets parent should compose child-owned extraction, conversion, and mutation"
    );
}
