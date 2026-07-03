#[test]
fn review_f5_font_asset_uses_typed_error_source() {
    let font = include_str!("../../../../../asset/assets/font.rs");
    let asset_assets_mod = include_str!("../../../../../asset/assets/mod.rs");
    let asset_mod = include_str!("../../../../../asset/mod.rs");
    let font_tests = include_str!("../../../../../asset/tests/assets/font.rs");
    let import_font_asset =
        include_str!("../../../../../asset/importer/ingest/import_font_asset/mod.rs");
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let font_doc = include_str!("../../../../../../../docs/zircon_runtime/asset/assets/font.md");

    for required in [
        "pub type FontAssetResult<T> = std::result::Result<T, FontAssetError>;",
        "pub enum FontAssetError",
        "Parse(#[source] toml::de::Error)",
        "pub fn from_toml_str(document: &str) -> FontAssetResult<Self>",
        "toml::from_str(document).map_err(FontAssetError::Parse)",
    ] {
        assert!(
            font.contains(required),
            "F5 font asset typed error owner should contain `{required}`"
        );
    }
    for forbidden in [
        "Parse(String)",
        "Result<Self, FontAssetError>",
        "error.to_string()",
    ] {
        assert!(
            !font.contains(forbidden),
            "font asset parsing should not keep lossy String error branch `{forbidden}`"
        );
    }
    for required in [
        "FontAssetError",
        "FontAssetResult",
        "font_asset_parse_reports_typed_toml_error_source",
        "FontAssetError::Parse(_)",
        "AssetImportError::Parse(format!(\"parse font toml: {error}\"))",
    ] {
        assert!(
            asset_assets_mod.contains(required)
                || asset_mod.contains(required)
                || font_tests.contains(required)
                || import_font_asset.contains(required),
            "font asset export/test/import surface should contain `{required}`"
        );
    }
    for doc_anchor in [
        "F5 font asset typed errors",
        "runtime_15_font_asset_typed_errors_static_passed_cargo_deferred",
        "review_f5_font_asset_uses_typed_error_source",
        "FontAssetError::Parse",
        "asset/assets/font.rs",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || font_doc.contains(doc_anchor),
            "F5 font asset docs should record `{doc_anchor}`"
        );
    }
}
