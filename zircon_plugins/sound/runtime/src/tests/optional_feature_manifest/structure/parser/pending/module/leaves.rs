// 源码结构守卫：固定可选功能解析器的状态与提交边界；运行清单语义由 parity 测试验证。
use super::super::super::super::sources::*;

#[test]
fn optional_feature_parser_pending_module_leaf_children_keep_finalize_ownership() {
    assert!(
        PARSER_PENDING_MODULE_SIGNATURE.contains("let name = name.take()?")
            && PARSER_PENDING_MODULE_APPEND.contains("parent.modules.push(module)"),
        "pending module signature and append children should keep leaf ownership"
    );
}
