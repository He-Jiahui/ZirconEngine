#[test]
fn review_f5_zshader_v2_replaces_user_shader_definitions() {
    let zshader = include_str!("../../../../../asset/assets/shader/zshader.rs");
    let shader_mod = include_str!("../../../../../asset/assets/shader/mod.rs");
    let asset_mod = include_str!("../../../../../asset/mod.rs");
    let import_shader_package =
        include_str!("../../../../../asset/importer/ingest/import_shader_package.rs");
    let shader_tests = include_str!("../../../../../asset/tests/assets/shader_readiness.rs");
    let review_findings = concat!(
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
        include_str!("../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
    );
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let shader_material_doc =
        include_str!("../../../../../../../docs/zircon_runtime/asset/zmeta-shader-material.md");

    for required in [
        "pub enum ZShaderDocumentV2",
        "pub struct ZShaderOptionDocument",
        "pub struct ShaderOptionAsset",
        "ForbiddenField",
    ] {
        assert!(
            zshader.contains(required),
            "F5 zshader v2 owner should contain `{required}`"
        );
    }
    for forbidden in [
        "pub struct ZShaderDocument",
        "pub type ZShaderDefinitionResult",
        "pub enum ZShaderDefinitionError",
        "pub struct ZShaderDefinitionValueDocument",
        "shader_definition_values",
        "Result<Vec<RenderShaderDefinitionValue>, String>",
        "Err(format!(",
    ] {
        assert!(
            !zshader.contains(forbidden),
            "zshader definition conversion should not keep String error branch `{forbidden}`"
        );
    }
    for required in [
        "ZShaderDocumentV2",
        "ZShaderV2Error",
        "ZShaderOptionDocument",
        "ShaderOptionAsset",
        "removed user-authored pipeline layout and shader_defs fields must be migrated",
    ] {
        assert!(
            shader_mod.contains(required)
                || asset_mod.contains(required)
                || shader_tests.contains(required)
                || import_shader_package.contains(required),
            "zshader export/test/import surface should contain `{required}`"
        );
    }
    for doc_anchor in [
        "`.zshader` v2",
        "shader_defs",
        "options",
        "asset/assets/shader/zshader.rs",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || shader_material_doc.contains(doc_anchor),
            "F5 zshader docs should record `{doc_anchor}`"
        );
    }
}
