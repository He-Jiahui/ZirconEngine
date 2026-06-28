#[test]
fn review_f5_ui_asset_documents_use_typed_errors_before_import_boundary() {
    let ui_assets = include_str!("../../../../asset/assets/ui.rs");
    let importer_error = include_str!("../../../../asset/importer/error.rs");
    let import_ui_zui_asset =
        include_str!("../../../../asset/importer/ingest/import_ui_zui_asset.rs");
    let ui_v2_document_import =
        include_str!("../../../../asset/importer/ingest/ui_v2_document_import.rs");
    let import_ui_theme_asset =
        include_str!("../../../../asset/importer/ingest/import_ui_theme_asset.rs");
    let import_ui_icon_asset =
        include_str!("../../../../asset/importer/ingest/import_ui_icon_asset.rs");
    let wrapper_tests = include_str!("../../../../asset/tests/assets/ui/wrappers.rs");
    let importer_tests = include_str!("../../../../asset/tests/assets/ui/importer.rs");
    let fixture_tests = include_str!("../../../../asset/tests/assets/ui/fixture_validation.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let module_doc =
        include_str!("../../../../../../docs/zircon_runtime/structure/module-convention.md");
    let ui_asset_doc = include_str!("../../../../../../docs/zircon_runtime/asset/assets/ui.md");
    let status_rows = include_str!(
        "../../plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs"
    );
    let status_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs"
    );
    let date_map = include_str!(
        "../../plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs"
    );

    for required in [
        "pub type UiAssetDocumentResult<T>",
        "pub type UiThemeAssetDocumentResult<T>",
        "pub type UiIconAssetDocumentResult<T>",
        "pub type UiV2AssetDocumentResult<T>",
        "Parse(#[from] UiAssetError)",
        "Parse(#[source] toml::de::Error)",
        "InvalidSourceUri {",
        "Parse(#[from] UiV2AssetError)",
        "ComponentRequiresZui",
        "UiAssetLoader::load_toml_str(document)?",
        "UiV2AssetLoader::load_toml_str(document)?",
    ] {
        assert!(
            ui_assets.contains(required),
            "UI asset typed-error owner should contain `{required}`"
        );
    }

    for forbidden in [
        "UiAssetDocumentError::Parse(error.to_string())",
        "UiThemeAssetDocumentError::Parse(error.to_string())",
        "UiIconAssetDocumentError::Parse(error.to_string())",
        "UiIconAssetDocumentError::Invalid(",
        "UiV2AssetDocumentError::Parse(error.to_string())",
    ] {
        assert!(
            !ui_assets.contains(forbidden),
            "UI asset wrappers should not keep lossy String branch `{forbidden}`"
        );
    }

    for required in [
        "UiDocument {",
        "UiV2Document {",
        "UiThemeDocument {",
        "UiIconDocument {",
        "source: UiThemeAssetDocumentError",
        "source: UiIconAssetDocumentError",
    ] {
        assert!(
            importer_error.contains(required),
            "AssetImportError should expose UI document typed source `{required}`"
        );
    }

    for (label, source) in [
        ("ZUI importer", import_ui_zui_asset),
        ("UI theme importer", import_ui_theme_asset),
        ("UI icon importer", import_ui_icon_asset),
    ] {
        for forbidden in [
            "AssetImportError::Parse(error.to_string())",
            "parse ui asset toml {}",
            "parse ui v2 asset toml {}",
            "parse .zui component asset {}",
            "parse ui theme asset {}",
            "parse ui icon asset {}",
            "unsupported or mismatched [asset.kind]",
        ] {
            assert!(
                !source.contains(forbidden),
                "{label} should not lossy-wrap UI document errors with `{forbidden}`"
            );
        }
    }

    for required in [
        "AssetImportError::UiV2Document",
        "source: source.into()",
        "UiV2AssetDocumentError::ComponentRequiresZui",
        "UiV2AssetKind::ThemeTokens",
    ] {
        assert!(
            import_ui_zui_asset.contains(required) || ui_v2_document_import.contains(required),
            "UI v2 importer path should contain typed branch `{required}`"
        );
    }
    for required in [
        "AssetImportError::UiV2Document",
        "AssetImportError::UiThemeDocument",
        "AssetImportError::UiIconDocument",
    ] {
        assert!(
            import_ui_zui_asset.contains(required)
                || import_ui_theme_asset.contains(required)
                || import_ui_icon_asset.contains(required),
            "UI specialized importer should preserve typed source `{required}`"
        );
    }

    for required in [
        "ui_asset_wrappers_preserve_typed_parse_sources",
        "ui_icon_asset_reports_typed_validation_errors",
        "importer_preserves_typed_ui_asset_document_parse_sources",
        "importer_preserves_typed_theme_and_icon_document_sources",
        "UiAssetError::ParseToml",
        "UiV2AssetError::ParseToml",
        "UiIconAssetDocumentError::InvalidSourceUri",
        "UiV2AssetDocumentError::ComponentRequiresZui",
    ] {
        assert!(
            wrapper_tests.contains(required)
                || importer_tests.contains(required)
                || fixture_tests.contains(required),
            "UI asset typed-error behavior tests should contain `{required}`"
        );
    }

    for doc_anchor in [
        "Runtime 15 F5 UI asset document typed errors",
        "runtime_15_ui_asset_document_typed_errors_static_passed_cargo_deferred",
        "review_f5_ui_asset_documents_use_typed_errors_before_import_boundary",
        "asset/assets/ui.rs",
        "asset/importer/ingest/import_ui_zui_asset.rs",
        "UiIconAssetDocumentError::InvalidSourceUri",
        "AssetImportError::UiIconDocument",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || module_doc.contains(doc_anchor)
                || ui_asset_doc.contains(doc_anchor)
                || status_rows.contains(doc_anchor)
                || status_map.contains(doc_anchor)
                || date_map.contains(doc_anchor),
            "F5 UI asset document typed-error docs/status should record `{doc_anchor}`"
        );
    }
}
