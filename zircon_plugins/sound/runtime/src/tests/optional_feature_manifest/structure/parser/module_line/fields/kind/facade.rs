use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_module_line_kind_facade_stays_structural() {
    assert!(
        PARSER_LINE_MODULE_KIND.contains("mod field;")
            && PARSER_LINE_MODULE_KIND.contains("mod state;")
            && PARSER_LINE_MODULE_KIND.contains("mod value;"),
        "module kind parent must remain split into field, state, and value children"
    );
    assert!(
        PARSER_LINE_MODULE_KIND.contains("field::module_kind_value")
            && PARSER_LINE_MODULE_KIND.contains("state::set_module_kind")
            && PARSER_LINE_MODULE_KIND.contains("value::module_kind_from_plugin_toml"),
        "module kind parent should compose child-owned extraction, mutation, and conversion"
    );
}
