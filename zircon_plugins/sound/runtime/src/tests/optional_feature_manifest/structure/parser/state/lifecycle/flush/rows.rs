// 源码结构守卫：固定可选功能解析器的状态与提交边界；运行清单语义由 parity 测试验证。
use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_state_flush_row_children_keep_pending_handoffs() {
    assert!(
        PARSER_STATE_FLUSH_DEPENDENCY.contains("push_optional_feature_dependency")
            && PARSER_STATE_FLUSH_DEPENDENCY.contains("current_dependency_plugin_id")
            && PARSER_STATE_FLUSH_MODULE.contains("push_optional_feature_module")
            && PARSER_STATE_FLUSH_MODULE.contains("current_module_target_modes")
            && PARSER_STATE_FLUSH_FEATURE.contains("push_optional_feature")
            && PARSER_STATE_FLUSH_FEATURE.contains("current_feature"),
        "parser state flush leaf children should own dependency, module, and feature row handoffs"
    );
}
