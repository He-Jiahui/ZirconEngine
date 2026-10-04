// 源码结构守卫：固定可选功能解析器的状态与提交边界；运行清单语义由 parity 测试验证。
use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_module_line_capabilities_state_child_mutates_pending_values() {
    assert!(
        PARSER_LINE_MODULE_CAPABILITIES_STATE.contains("*capabilities = values"),
        "module capabilities state child should own pending capability-list mutation"
    );
}
