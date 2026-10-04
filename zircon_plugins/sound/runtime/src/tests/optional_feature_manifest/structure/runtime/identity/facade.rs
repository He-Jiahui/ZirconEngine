// 源码结构守卫：固定可选功能运行时投影的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::sources::*;

#[test]
fn optional_feature_runtime_identity_facade_stays_structural() {
    assert!(
        RUNTIME_IDENTITY_ROOT.contains("mod display_name;")
            && RUNTIME_IDENTITY_ROOT.contains("mod id;")
            && RUNTIME_IDENTITY_ROOT.contains("mod owner_plugin;"),
        "runtime identity parent must remain a structural child-module owner"
    );
    assert!(
        !RUNTIME_IDENTITY_ROOT.contains("fn feature_id")
            && !RUNTIME_IDENTITY_ROOT.contains("fn feature_display_name")
            && !RUNTIME_IDENTITY_ROOT.contains("fn feature_owner_plugin_id"),
        "runtime identity parent must not own feature identity forwarding bodies"
    );
    assert!(
        RUNTIME_IDENTITY_ROOT.contains("use display_name::feature_display_name")
            && RUNTIME_IDENTITY_ROOT.contains("use id::feature_id")
            && RUNTIME_IDENTITY_ROOT.contains("use owner_plugin::feature_owner_plugin_id"),
        "runtime identity parent should expose identity projection helpers through child re-exports"
    );
}
