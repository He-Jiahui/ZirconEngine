// 源码结构守卫：固定可选功能测试类型的声明归属；运行清单语义由 parity 测试验证。
use super::super::sources::*;

#[test]
fn optional_feature_dependency_signature_declaration_stays_split_from_facade() {
    assert!(
        TYPES_DEPENDENCY_SIGNATURE.contains("type OptionalFeatureDependencySignature")
            && TYPES_DEPENDENCY_SIGNATURE.contains("(String, String, bool)"),
        "dependency signature child should own the dependency tuple declaration"
    );
}
