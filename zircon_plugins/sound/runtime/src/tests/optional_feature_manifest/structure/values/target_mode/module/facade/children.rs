// 源码结构守卫：固定可选功能值转换的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::super::super::sources::*;

#[test]
fn optional_feature_target_mode_module_facade_declares_entry_child() {
    assert!(
        TARGET_MODE_MODULE.contains("mod entry;"),
        "module target-mode projection parent should declare the entry owner"
    );
}
