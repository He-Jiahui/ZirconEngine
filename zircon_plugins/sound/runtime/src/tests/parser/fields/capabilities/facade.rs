// 源码结构守卫：固定可选功能解析器的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_module_line_capabilities_facade_stays_structural() {
    assert!(
        PARSER_LINE_MODULE_CAPABILITIES.contains("mod field;")
            && PARSER_LINE_MODULE_CAPABILITIES.contains("mod state;")
            && PARSER_LINE_MODULE_CAPABILITIES.contains("mod values;"),
        "module capabilities parent must remain split into field, state, and values children"
    );
    assert!(
        PARSER_LINE_MODULE_CAPABILITIES.contains("field::module_capabilities_value")
            && PARSER_LINE_MODULE_CAPABILITIES
                .contains("values::module_capabilities_from_plugin_toml")
            && PARSER_LINE_MODULE_CAPABILITIES.contains("state::set_module_capabilities"),
        "module capabilities parent should compose child-owned extraction, conversion, and mutation"
    );
}
