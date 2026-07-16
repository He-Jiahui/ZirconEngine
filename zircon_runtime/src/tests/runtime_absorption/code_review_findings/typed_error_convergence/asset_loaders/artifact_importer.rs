#[test]
fn review_f7_asset_artifact_errors_use_asset_import_error_sources() {
    let importer_error = include_str!("../../../../../asset/importer/error.rs");
    let cache_payload = include_str!("../../../../../asset/artifact/cache_payload.rs");
    let json_value = include_str!("../../../../../asset/artifact/cache_payload/json_value.rs");
    let cache_payload_ui = include_str!("../../../../../asset/artifact/cache_payload/ui.rs");
    let toml_value = include_str!("../../../../../asset/artifact/cache_payload/toml_value.rs");
    let artifact_store = include_str!("../../../../../asset/artifact/store.rs");
    let importer_tests =
        include_str!("../../../../../asset/tests/assets/importer/registry_errors.rs");
    let review_findings = concat!(
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md"),
        include_str!("../../../../../../../docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
    );
    let runtime_04_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let artifact_doc = include_str!("../../../../../../../docs/zircon_runtime/asset/artifact.md");

    for forbidden in [
        "Registry(String)",
        "Self::Registry(error.to_string())",
        "impl From<AssetImporterRegistryError> for AssetImportError",
    ] {
        assert!(
            !importer_error.contains(forbidden),
            "F7 should not preserve lossy registry error conversion `{forbidden}`"
        );
    }
    for required in [
        "Registry(#[from] AssetImporterRegistryError)",
        "TomlSerialize {",
        "TomlDeserialize {",
        "CachedTomlDatetime {",
        "UiDocument {",
        "UiV2Document {",
        "ArtifactCacheSerialize(#[source] bincode::Error)",
        "ArtifactCacheDeserialize(#[source] bincode::Error)",
        "CachedJsonNonFiniteNumber {",
        "CachedJsonNumberParse {",
    ] {
        assert!(
            importer_error.contains(required),
            "F7 AssetImportError should expose typed source anchor `{required}`"
        );
    }

    for forbidden in [
        "pub(super) fn from_imported(asset: &ImportedAsset) -> Result<Self, String>",
        "pub(super) fn into_imported(self) -> Result<ImportedAsset, String>",
        "fn into_asset(self) -> Result<MaterialAsset, String>",
        "fn into_asset(self) -> Result<ShaderAsset, String>",
        "fn into_asset(self) -> Result<ShaderMaterialPropertyAsset, String>",
        "format!(\"serialize ui asset document cache",
        "format!(\"deserialize ui layout document cache",
        "format!(\"deserialize ui v2 view document cache",
    ] {
        assert!(
            !cache_payload.contains(forbidden),
            "F7 cache payload should not keep String error/string-format anchor `{forbidden}`"
        );
    }
    for required in [
        "use crate::asset::{",
        "AssetImportError,",
        "pub(super) fn from_imported(asset: &ImportedAsset) -> Result<Self, AssetImportError>",
        "pub(super) fn into_imported(self) -> Result<ImportedAsset, AssetImportError>",
        "ArtifactCacheUiAssetDocument::from_document",
        "ArtifactCacheUiV2AssetDocument::from_document",
    ] {
        assert!(
            cache_payload.contains(required),
            "F7 cache payload should use AssetImportError anchor `{required}`"
        );
    }
    for forbidden in [".expect(", ".unwrap()"] {
        assert!(
            !json_value.contains(forbidden),
            "F7 JSON cache conversion should not keep panic branch `{forbidden}`"
        );
    }
    for required in [
        "Result<serde_json::Value, AssetImportError>",
        "AssetImportError::CachedJsonNonFiniteNumber",
        "AssetImportError::CachedJsonNumberParse",
        "collect::<Result<Vec<_>, _>>()?",
    ] {
        assert!(
            json_value.contains(required),
            "F7 JSON cache conversion should preserve typed error anchor `{required}`"
        );
    }
    for required in [
        "AssetImportError::TomlSerialize",
        "AssetImportError::UiDocument",
        "AssetImportError::UiV2Document",
    ] {
        assert!(
            cache_payload_ui.contains(required),
            "F7 UI cache payload should preserve AssetImportError source anchor `{required}`"
        );
    }

    assert!(
        toml_value.contains("Result<toml::Value, AssetImportError>")
            && toml_value.contains("AssetImportError::CachedTomlDatetime")
            && !toml_value.contains("format!(\"invalid cached TOML datetime"),
        "F7 TOML cache conversion should report typed cached datetime errors"
    );
    for required in [
        "map_err(AssetImportError::ArtifactCacheSerialize)",
        "map_err(AssetImportError::ArtifactCacheDeserialize)",
        "let cache_asset = ArtifactCacheAsset::from_imported(asset)?;",
        "let asset = cache_asset.into_imported()?;",
    ] {
        assert!(
            artifact_store.contains(required),
            "F7 artifact store should preserve typed source anchor `{required}`"
        );
    }
    assert!(
        !artifact_store
            .contains("map_err(|error| AssetImportError::Parse(format!(\"serialize artifact cache")
            && !artifact_store.contains(
                "map_err(|error| AssetImportError::Parse(format!(\"deserialize artifact cache"
            ),
        "F7 artifact store should not lossy-wrap cache conversion sources in Parse(String)"
    );
    assert!(
        importer_tests.contains("asset_import_error_preserves_registry_error_source")
            && importer_tests.contains(
                "AssetImportError::Registry(AssetImporterRegistryError::DuplicateMatcher"
            ),
        "F7 should keep behavior coverage for typed registry error preservation"
    );

    for doc_anchor in [
        "F7 asset artifact/importer typed errors",
        "asset_artifact_importer_typed_errors_coremin_passed",
        "review_f7_asset_artifact_errors_use_asset_import_error_sources",
        "asset_import_error_preserves_registry_error_source",
        "AssetImportError::CachedTomlDatetime",
        "f5_f6_f7_typed_error_top_row_closed_status_static_passed_cargo_deferred",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_04_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || artifact_doc.contains(doc_anchor),
            "F7 docs should record `{doc_anchor}`"
        );
    }
    let f7_row = review_findings
        .lines()
        .find(|line| line.starts_with("| F7 |"))
        .expect("F7 review findings top row");
    assert!(
        f7_row.contains("asset artifact/importer") && f7_row.ends_with("| Runtime 04 |"),
        "F7 overview row should keep only the finding and Runtime 04 owner"
    );
    assert!(
        review_findings
            .contains("f5_f6_f7_typed_error_top_row_closed_status_static_passed_cargo_deferred"),
        "F7 numbered output should record typed-error review closed status"
    );
}
