// 源码结构守卫：固定可选功能值转换的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_string_feature_facade_declares_field_children() {
    assert!(
        STRING_FEATURE.contains("mod display_name;")
            && STRING_FEATURE.contains("mod id;")
            && STRING_FEATURE.contains("mod owner_plugin;"),
        "feature string domain should declare field-owner children"
    );
}
