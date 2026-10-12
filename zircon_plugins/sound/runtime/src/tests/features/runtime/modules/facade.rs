// 源码结构守卫：固定可选功能运行时投影的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::super::sources::*;

#[test]
fn optional_feature_runtime_module_collection_facade_stays_structural() {
    assert!(
        RUNTIME_MODULES_ROOT.contains("mod entry;")
            && RUNTIME_MODULES_ROOT.contains("mod ordering;")
            && RUNTIME_MODULES_ROOT.contains("mod projection;")
            && RUNTIME_MODULES_ROOT.contains("mod signature;"),
        "runtime modules parent must remain a structural child-module owner"
    );
    assert!(
        !RUNTIME_MODULES_ROOT.contains("fn module_signatures"),
        "runtime modules parent must not own projection and ordering composition"
    );
    assert!(
        RUNTIME_MODULES_ROOT.contains("use entry::module_signatures"),
        "runtime modules parent should expose the child-owned entry point"
    );
}
