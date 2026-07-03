#[test]
fn review_f5_asset_meta_uses_typed_error() {
    let meta = include_str!("../../../../../asset/project/meta.rs");
    let asset_mod = include_str!("../../../../../asset/mod.rs");
    let importer_doc = include_str!("../../../../../../../docs/zircon_runtime/asset/importer.md");
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");

    for required in [
        "pub type AssetMetaResult<T> = std::result::Result<T, AssetMetaError>;",
        "pub enum AssetMetaError",
        "UnsupportedFormatVersion { found: u32, supported: u32 }",
        "fn migrate_to_current(&mut self) -> AssetMetaResult<()>",
        "asset_meta_migration_reports_typed_future_version_error",
    ] {
        assert!(
            meta.contains(required),
            "F5 asset meta typed error owner should contain `{required}`"
        );
    }
    for forbidden in ["Result<(), String>", "Err(format!("] {
        assert!(
            !meta.contains(forbidden),
            "asset meta migration should not keep String error branch `{forbidden}`"
        );
    }
    assert!(
        asset_mod.contains("AssetMetaError") && asset_mod.contains("AssetMetaResult"),
        "asset facade should export AssetMetaError/AssetMetaResult"
    );
    for doc_anchor in [
        "F5 asset meta typed errors",
        "runtime_15_asset_meta_typed_errors_static_passed_cargo_deferred",
        "review_f5_asset_meta_uses_typed_error",
        "AssetMetaError::UnsupportedFormatVersion",
        "asset/project/meta.rs",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || importer_doc.contains(doc_anchor),
            "F5 asset meta docs should record `{doc_anchor}`"
        );
    }
}
