#[test]
fn review_f5_asset_authoring_uses_typed_error() {
    let authoring = include_str!("../../../../asset/assets/authoring.rs");
    let asset_assets_mod = include_str!("../../../../asset/assets/mod.rs");
    let asset_mod = include_str!("../../../../asset/mod.rs");
    let import_authoring_asset =
        include_str!("../../../../asset/importer/ingest/import_authoring_asset.rs");
    let asset_authoring_tests = include_str!("../../../../asset/tests/assets/authoring.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let authoring_doc =
        include_str!("../../../../../../docs/zircon_runtime/asset/assets/authoring.md");

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

#[test]
fn review_f5_navigation_asset_uses_typed_error() {
    let navigation = include_str!("../../../../asset/assets/navigation.rs");
    let asset_assets_mod = include_str!("../../../../asset/assets/mod.rs");
    let asset_mod = include_str!("../../../../asset/mod.rs");
    let navigation_tests = include_str!("../../../../asset/tests/assets/navigation.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let navigation_doc =
        include_str!("../../../../../../docs/zircon_runtime/asset/assets/navigation.md");

    for required in [
        "pub type NavigationAssetResult<T> = std::result::Result<T, NavigationAssetError>;",
        "pub enum NavigationAssetError",
        "Serialize(#[source] bincode::Error)",
        "Deserialize(#[source] bincode::Error)",
        "pub fn to_bytes(&self) -> NavigationAssetResult<Vec<u8>>",
        "pub fn from_bytes(bytes: &[u8]) -> NavigationAssetResult<Self>",
    ] {
        assert!(
            navigation.contains(required),
            "F5 navigation asset typed error owner should contain `{required}`"
        );
    }
    for forbidden in [
        "Result<Vec<u8>, String>",
        "Result<Self, String>",
        "error.to_string()",
    ] {
        assert!(
            !navigation.contains(forbidden),
            "navigation asset binary helpers should not keep String error branch `{forbidden}`"
        );
    }
    for required in [
        "NavigationAssetError",
        "NavigationAssetResult",
        "navmesh_binary_roundtrip_reports_typed_errors",
        "NavigationAssetError::Deserialize(_)",
    ] {
        assert!(
            asset_assets_mod.contains(required)
                || asset_mod.contains(required)
                || navigation_tests.contains(required),
            "navigation asset export/test surface should contain `{required}`"
        );
    }
    for doc_anchor in [
        "F5 navigation asset typed errors",
        "runtime_15_navigation_asset_typed_errors_static_passed_cargo_deferred",
        "review_f5_navigation_asset_uses_typed_error",
        "NavigationAssetError",
        "asset/assets/navigation.rs",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || navigation_doc.contains(doc_anchor),
            "F5 navigation asset docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f5_font_asset_uses_typed_error_source() {
    let font = include_str!("../../../../asset/assets/font.rs");
    let asset_assets_mod = include_str!("../../../../asset/assets/mod.rs");
    let asset_mod = include_str!("../../../../asset/mod.rs");
    let font_tests = include_str!("../../../../asset/tests/assets/font.rs");
    let import_font_asset =
        include_str!("../../../../asset/importer/ingest/import_font_asset/mod.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let font_doc = include_str!("../../../../../../docs/zircon_runtime/asset/assets/font.md");

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

#[test]
fn review_f5_sound_asset_uses_typed_error() {
    let sound = include_str!("../../../../asset/assets/sound.rs");
    let asset_assets_mod = include_str!("../../../../asset/assets/mod.rs");
    let asset_mod = include_str!("../../../../asset/mod.rs");
    let sound_tests = include_str!("../../../../asset/tests/assets/sound.rs");
    let import_sound_asset = include_str!("../../../../asset/importer/ingest/import_sound.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let sound_doc = include_str!("../../../../../../docs/zircon_runtime/asset/assets/sound.md");
    let importer_doc = include_str!("../../../../../../docs/zircon_runtime/asset/importer.md");

    for required in [
        "pub type SoundAssetResult<T> = std::result::Result<T, SoundAssetError>;",
        "pub enum SoundAssetError",
        "UnsupportedSpeakerMaskBits",
        "UnsupportedBitsPerSample",
        "pub fn from_wav_bytes(uri: &AssetUri, bytes: &[u8]) -> SoundAssetResult<Self>",
        "fn parse_format_chunk(bytes: &[u8]) -> SoundAssetResult<WavFormat>",
        "fn decode_samples(format: &WavFormat, data: &[u8]) -> SoundAssetResult<Vec<f32>>",
    ] {
        assert!(
            sound.contains(required),
            "F5 sound asset typed error owner should contain `{required}`"
        );
    }
    for forbidden in [
        "Result<Self, String>",
        "Result<WavFormat, String>",
        "Result<Vec<f32>, String>",
        "Result<SoundChannelLayout, String>",
        "Result<(), String>",
        "Err(format!(",
        ".to_string()",
    ] {
        assert!(
            !sound.contains(forbidden),
            "sound WAV parsing should not keep lossy String error branch `{forbidden}`"
        );
    }
    for required in [
        "SoundAssetError",
        "SoundAssetResult",
        "sound_asset_wav_parse_reports_typed_error_variants",
        "SoundAssetError::UnsupportedSpeakerMaskBits",
        "SoundAssetError::UnsupportedBitsPerSample",
        "AssetImportError::Parse(format!(\"decode wav {}: {error}\"",
    ] {
        assert!(
            asset_assets_mod.contains(required)
                || asset_mod.contains(required)
                || sound_tests.contains(required)
                || import_sound_asset.contains(required),
            "sound asset export/test/import surface should contain `{required}`"
        );
    }
    for doc_anchor in [
        "F5 sound asset typed errors",
        "runtime_15_sound_asset_typed_errors_static_passed_cargo_deferred",
        "review_f5_sound_asset_uses_typed_error",
        "SoundAssetError::UnsupportedSpeakerMaskBits",
        "asset/assets/sound.rs",
        "asset/tests/assets/sound.rs",
    ] {
        assert!(
            review_findings.contains(doc_anchor)
                || runtime_15_plan.contains(doc_anchor)
                || runtime_index.contains(doc_anchor)
                || convention.contains(doc_anchor)
                || sound_doc.contains(doc_anchor)
                || importer_doc.contains(doc_anchor),
            "F5 sound asset docs should record `{doc_anchor}`"
        );
    }
}

#[test]
fn review_f5_zshader_definition_values_use_typed_error() {
    let zshader = include_str!("../../../../asset/assets/shader/zshader.rs");
    let shader_mod = include_str!("../../../../asset/assets/shader/mod.rs");
    let asset_mod = include_str!("../../../../asset/mod.rs");
    let import_shader_package =
        include_str!("../../../../asset/importer/ingest/import_shader_package.rs");
    let shader_tests = include_str!("../../../../asset/tests/assets/shader_readiness.rs");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");
    let shader_material_doc =
        include_str!("../../../../../../docs/zircon_runtime/asset/zmeta-shader-material.md");

    for required in [
        "pub type ZShaderDefinitionResult<T> = std::result::Result<T, ZShaderDefinitionError>;",
        "pub enum ZShaderDefinitionError",
        "BoolValue",
        "IntValue",
        "UintValue",
        "UnsupportedKind { name: String, kind: String }",
        ") -> ZShaderDefinitionResult<Vec<RenderShaderDefinitionValue>>",
        ") -> ZShaderDefinitionResult<RenderShaderDefinitionValue>",
    ] {
        assert!(
            zshader.contains(required),
            "F5 zshader typed error owner should contain `{required}`"
        );
    }
    for forbidden in [
        "Result<Vec<RenderShaderDefinitionValue>, String>",
        "Err(format!(",
    ] {
        assert!(
            !zshader.contains(forbidden),
            "zshader definition conversion should not keep String error branch `{forbidden}`"
        );
    }
    for required in [
        "ZShaderDefinitionError",
        "ZShaderDefinitionResult",
        "zshader_definition_values_report_typed_authoring_errors",
        "ZShaderDefinitionError::UnsupportedKind",
        "AssetImportError::Parse(format!(\"parse zshader shader_def_values: {error}\"))",
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
        "F5 zshader definition typed errors",
        "runtime_15_zshader_definition_typed_errors_static_passed_cargo_deferred",
        "review_f5_zshader_definition_values_use_typed_error",
        "ZShaderDefinitionError::UnsupportedKind",
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

#[test]
fn review_f5_asset_meta_uses_typed_error() {
    let meta = include_str!("../../../../asset/project/meta.rs");
    let asset_mod = include_str!("../../../../asset/mod.rs");
    let importer_doc = include_str!("../../../../../../docs/zircon_runtime/asset/importer.md");
    let review_findings =
        include_str!("../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../docs/plans/engine-code-structure-convention.md");

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
