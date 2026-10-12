// 源码结构守卫：固定可选功能测试类型的声明归属；运行清单语义由 parity 测试验证。
use super::super::sources::*;

#[test]
fn optional_feature_module_signature_declaration_stays_split_from_facade() {
    assert!(
        TYPES_MODULE_SIGNATURE.contains("type OptionalFeatureModuleSignature")
            && TYPES_MODULE_SIGNATURE.contains("PluginModuleKind")
            && TYPES_MODULE_SIGNATURE.contains("RuntimeTargetMode"),
        "module signature child should own the module tuple declaration"
    );
}
