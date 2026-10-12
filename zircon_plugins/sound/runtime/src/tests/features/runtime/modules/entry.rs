// 源码结构守卫：固定可选功能运行时投影的子层交接；运行清单语义由 parity 测试验证。
use super::super::super::super::sources::*;

#[test]
fn optional_feature_runtime_module_collection_entry_composes_projection_and_ordering() {
    assert!(
        RUNTIME_MODULES_ENTRY.contains("pub(in super::super) fn module_signatures")
            && RUNTIME_MODULES_ENTRY
                .contains("super::projection::project_module_signatures(feature)")
            && RUNTIME_MODULES_ENTRY
                .contains("super::ordering::sort_module_signatures(&mut modules)"),
        "runtime modules entry child should be facade-visible and compose projection and ordering"
    );
}
