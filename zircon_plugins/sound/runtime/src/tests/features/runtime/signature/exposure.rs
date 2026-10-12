// 源码结构守卫：固定可选功能运行时投影的声明归属；运行清单语义由 parity 测试验证。
use super::super::super::sources::*;

#[test]
fn optional_feature_runtime_signature_exposure_stays_on_runtime_facade() {
    assert!(
        RUNTIME_ROOT.contains("mod capabilities;")
            && RUNTIME_ROOT.contains("mod defaults;")
            && RUNTIME_ROOT.contains("mod dependencies;")
            && RUNTIME_ROOT.contains("mod identity;")
            && RUNTIME_ROOT.contains("mod modules;")
            && RUNTIME_ROOT.contains("mod signature;"),
        "runtime parent must remain a structural child-module owner"
    );
    assert!(
        !RUNTIME_ROOT.contains("StaticOptionalFeatureManifest")
            && !RUNTIME_ROOT.contains("id: identity::")
            && !RUNTIME_ROOT.contains("capabilities: capabilities::")
            && !RUNTIME_ROOT.contains("modules: modules::"),
        "runtime parent must not own full optional-feature signature assembly"
    );
    assert!(
        RUNTIME_ROOT.contains("use signature::optional_feature_signature"),
        "runtime parent should expose signature assembly through the signature child re-export"
    );
}
