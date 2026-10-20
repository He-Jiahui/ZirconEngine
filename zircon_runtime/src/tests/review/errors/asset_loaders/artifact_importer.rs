//! 代码审查回归护栏核对已迁移接口、错误边界、插件入口及镜像状态。以结果断言检查当前接口或源码快照对应的边界。
use crate::tests::runtime_absorption::structure_convention::fixed_history_archive_source as history;
use crate::tests::runtime_absorption::structure_convention::rust_source_view::production_section;
#[test]
fn review_f7_asset_artifact_errors_use_asset_import_error_sources() {
    let importer_error = include_str!("../../../../asset/importer/error.rs");
    let cache_payload = include_str!("../../../../asset/artifact/cache_payload.rs");
    let json_value = include_str!("../../../../asset/artifact/cache_payload/json_value.rs");
    let cache_payload_ui = include_str!("../../../../asset/artifact/cache_payload/ui.rs");
    let toml_value = include_str!("../../../../asset/artifact/cache_payload/toml_value.rs");
    let artifact_store = include_str!("../../../../asset/artifact/store.rs");
    let importer_tests = include_str!("../../../../asset/tests/assets/importer/registry_errors.rs");
    let review_findings = crate::runtime_test_history_concat!(
        include_str!("../../../../../../docs/architecture/runtime-architecture-review-m0.md"),
        history("docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md")
    );
    let runtime_04_plan = include_str!(
        "../../../../../../docs/testing-and-validation/runtime-asset-validation-worklist.md"
    );
    let runtime_index = include_str!(
        "../../../../../../docs/crates/zircon_runtime/structure/runtime-absorption-evidence.md"
    );
    let convention =
        include_str!("../../../../../../docs/testing-and-validation/development-conventions.md");
    let artifact_doc =
        include_str!("../../../../../../docs/crates/zircon_runtime/asset/artifact.md");

    let codec_error = include_str!("../../../../asset/codec_error.rs");
    let asset_mod = include_str!("../../../../asset/mod.rs");
    for required in [
        "#[cfg(feature = \"native-host\")]\npub(crate) use crate::asset::AssetImportError as AssetCodecError;",
        "#[cfg(not(feature = \"native-host\"))]",
        "enum AssetCodecError",
        "CachedJsonNonFiniteNumber",
        "CachedJsonNumberParse",
        "CachedTomlDatetime",
        "TomlSerialize",
        "UiDocument",
        "UiV2Document",
    ] {
        assert!(codec_error.contains(required), "current asset/codec_error.rs must retain {required}");
    }
    assert!(
        asset_mod
            .replace("\r\n", "\n")
            .contains("#[path = \"codec_error.rs\"]\nmod codec_error;"),
        "exact current module mount in asset/mod.rs"
    );
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
        "use zircon_runtime_interface::serialization::{PositionalDecodeError, PositionalEncodeError};",
        "ArtifactCacheSerialize(#[source] PositionalEncodeError)",
        "ArtifactCacheDeserialize(#[source] PositionalDecodeError)",
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
        "AssetCodecError,",
        "pub(super) fn from_imported(asset: &ImportedAsset) -> Result<Self, AssetCodecError>",
        "pub(super) fn into_imported(self) -> Result<ImportedAsset, AssetCodecError>",
        "ArtifactCacheUiAssetDocument::from_document",
        "ArtifactCacheUiV2AssetDocument::from_document",
    ] {
        assert!(
            cache_payload.contains(required),
            "F7 cache payload should use AssetCodecError anchor `{required}`"
        );
    }
    for forbidden in [".expect(", ".unwrap()"] {
        assert!(
            !production_section(json_value).contains(forbidden),
            "F7 JSON cache conversion should not keep panic branch `{forbidden}`"
        );
    }
    for required in [
        "Result<serde_json::Value, AssetCodecError>",
        "AssetCodecError::CachedJsonNonFiniteNumber",
        "AssetCodecError::CachedJsonNumberParse",
        "collect::<Result<Vec<_>, _>>()?",
    ] {
        assert!(
            json_value.contains(required),
            "F7 JSON cache conversion should preserve typed error anchor `{required}`"
        );
    }
    for required in [
        "AssetCodecError::TomlSerialize",
        "AssetCodecError::UiDocument",
        "AssetCodecError::UiV2Document",
    ] {
        assert!(
            cache_payload_ui.contains(required),
            "F7 UI cache payload should preserve AssetCodecError source anchor `{required}`"
        );
    }

    assert!(
        toml_value.contains("Result<toml::Value, AssetCodecError>")
            && toml_value.contains("AssetCodecError::CachedTomlDatetime")
            && !toml_value.contains("format!(\"invalid cached TOML datetime"),
        "F7 TOML cache conversion should report typed cached datetime errors"
    );
    for required in [
        "map_err(AssetImportError::ArtifactCacheSerialize)",
        "map_err(AssetImportError::ArtifactCacheDeserialize)",
        "let cache_asset = super::cache_payload::from_imported_for_write(asset)?;",
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
