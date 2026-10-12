// 源码结构守卫：固定可选功能解析器的字段或值的归属；运行清单语义由 parity 测试验证。
use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_module_line_targets_modes_child_converts_typed_list() {
    assert!(
        PARSER_LINE_MODULE_TARGETS_MODES
            .contains("module_target_mode_list_from_plugin_toml(value)"),
        "module targets modes child should own typed target-mode list conversion"
    );
}
