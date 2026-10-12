// 源码结构守卫：固定可选功能值转换的字段或值的归属；运行清单语义由 parity 测试验证。
use super::super::super::sources::*;

#[test]
fn optional_feature_boolean_dependency_projection_calls_raw_parser_owner() {
    assert!(
        BOOLEAN_DEPENDENCY.contains("super::raw::bool_from_plugin_toml(value)"),
        "dependency boolean projection should call the raw parser from its child owner"
    );
}
