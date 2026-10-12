// 源码结构守卫：固定可选功能测试类型的声明归属；运行清单语义由 parity 测试验证。
use super::super::sources::*;

#[test]
fn optional_feature_static_manifest_declaration_stays_split_from_facade() {
    assert!(
        TYPES_STATIC_MANIFEST.contains("struct StaticOptionalFeatureManifest")
            && TYPES_STATIC_MANIFEST.contains("fn id(&self) -> &str")
            && TYPES_STATIC_MANIFEST.contains("enabled_by_default: bool")
            && TYPES_STATIC_MANIFEST.contains("dependencies:")
            && TYPES_STATIC_MANIFEST.contains("modules:"),
        "static manifest child should own comparison DTO declaration and accessor"
    );
}
