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
