// 源码结构守卫：固定可选功能值转换的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::super::sources::*;

#[test]
fn optional_feature_boolean_facade_declares_value_children() {
    assert!(
        BOOLEAN_ROOT.contains("mod dependency;")
            && BOOLEAN_ROOT.contains("mod feature;")
            && BOOLEAN_ROOT.contains("mod raw;"),
        "boolean parent must remain a structural child-module owner"
    );
}
