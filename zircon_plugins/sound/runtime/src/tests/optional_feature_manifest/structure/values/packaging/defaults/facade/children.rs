// 源码结构守卫：固定可选功能值转换的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_packaging_defaults_facade_declares_entry_child() {
    assert!(
        PACKAGING_DEFAULTS.contains("mod entry;"),
        "default packaging projection parent should declare the entry owner"
    );
}
