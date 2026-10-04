// 源码结构守卫：固定可选功能运行时投影的模块职责和重导出边界；运行清单语义由 parity 测试验证。
use super::super::super::sources::*;

#[test]
fn optional_feature_runtime_defaults_facade_stays_structural() {
    assert!(
        RUNTIME_DEFAULTS_ROOT.contains("mod enabled;")
            && RUNTIME_DEFAULTS_ROOT.contains("mod packaging;"),
        "runtime defaults parent must remain a structural child-module owner"
    );
    assert!(
        !RUNTIME_DEFAULTS_ROOT.contains("fn default_packaging")
            && !RUNTIME_DEFAULTS_ROOT.contains("fn enabled_by_default"),
        "runtime defaults parent must not own default forwarding bodies"
    );
    assert!(
        RUNTIME_DEFAULTS_ROOT.contains("use enabled::enabled_by_default")
            && RUNTIME_DEFAULTS_ROOT.contains("use packaging::default_packaging"),
        "runtime defaults parent should expose default projection helpers through child re-exports"
    );
}
