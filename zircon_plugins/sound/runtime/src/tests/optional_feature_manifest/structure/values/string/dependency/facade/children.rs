// 源码结构守卫：固定可选功能值转换的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_string_dependency_facade_declares_field_children() {
    assert!(
        STRING_DEPENDENCY.contains("mod capability;")
            && STRING_DEPENDENCY.contains("mod plugin_id;"),
        "dependency string domain should stay structural and re-export field owners"
    );
}
