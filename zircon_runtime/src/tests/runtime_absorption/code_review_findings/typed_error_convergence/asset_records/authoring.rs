#[test]
fn review_f5_asset_authoring_uses_typed_error() {
    let authoring = include_str!("../../../../../asset/assets/authoring.rs");
    let asset_assets_mod = include_str!("../../../../../asset/assets/mod.rs");
    let asset_mod = include_str!("../../../../../asset/mod.rs");
    let import_authoring_asset =
        include_str!("../../../../../asset/importer/ingest/import_authoring_asset.rs");
    let asset_authoring_tests = include_str!("../../../../../asset/tests/assets/authoring.rs");
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let authoring_doc =
        include_str!("../../../../../../../docs/zircon_runtime/asset/assets/authoring.md");

    for required in [
        "pub type AssetAuthoringResult<T> = std::result::Result<T, AssetAuthoringError>;",
        "pub enum AssetAuthoringError",
        "TerrainSampleCount",
        "TileMapLayerTileCount",
        "MaterialGraphMissingOutput",
        "pub fn validate_dimensions(&self) -> AssetAuthoringResult<()>",
        "pub fn validate_layers(&self) -> AssetAuthoringResult<()>",
        "pub fn validate_output_node(&self) -> AssetAuthoringResult<()>",
    ] {
        assert!(
            authoring.contains(required),
            "F5 asset authoring typed error owner should contain `{required}`"
        );
    }
    for forbidden in ["Result<(), String>", "Err(format!("] {
        assert!(
            !authoring.contains(forbidden),
            "asset authoring validation should not keep String error branch `{forbidden}`"
        );
    }
    for required in [
        "AssetAuthoringError",
        "AssetAuthoringResult",
        "asset_authoring_parse_error",
        "AssetImportError::Parse(error.to_string())",
        "authoring_asset_validation_reports_typed_errors",
    ] {
        assert!(
            asset_assets_mod.contains(required)
                || asset_mod.contains(required)
                || import_authoring_asset.contains(required)
                || asset_authoring_tests.contains(required),
            "asset authoring import/test surface should contain `{required}`"
        );
    }
    for doc_anchor in [
        "F5 asset authoring typed errors",
        "runtime_15_asset_authoring_typed_errors_static_passed_cargo_deferred",
        "review_f5_asset_authoring_uses_typed_error",
        "AssetAuthoringError",
        "asset/assets/authoring.rs",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || authoring_doc.contains(doc_anchor),
            "F5 asset authoring docs should record `{doc_anchor}`"
        );
    }
}
