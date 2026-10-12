// 源码结构守卫：固定可选功能测试类型的状态与提交边界；运行清单语义由 parity 测试验证。
use super::super::sources::*;

#[test]
fn optional_feature_pending_manifest_declaration_stays_split_from_facade() {
    assert!(
        TYPES_PENDING_MANIFEST.contains("struct PendingOptionalFeatureManifest")
            && TYPES_PENDING_MANIFEST.contains("id: Option<String>")
            && TYPES_PENDING_MANIFEST.contains("dependencies:")
            && TYPES_PENDING_MANIFEST.contains("modules:"),
        "pending manifest child should own scanner DTO declaration"
    );
}
