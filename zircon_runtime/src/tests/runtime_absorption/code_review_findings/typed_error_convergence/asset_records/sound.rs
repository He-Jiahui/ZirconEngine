#[test]
fn review_f5_sound_asset_uses_typed_error() {
    let sound = include_str!("../../../../../asset/assets/sound.rs");
    let asset_assets_mod = include_str!("../../../../../asset/assets/mod.rs");
    let asset_mod = include_str!("../../../../../asset/mod.rs");
    let sound_tests = include_str!("../../../../../asset/tests/assets/sound.rs");
    let import_sound_asset = include_str!("../../../../../asset/importer/ingest/import_sound.rs");
    let review_findings =
        include_str!("../../../../../../../docs/plans/engine-code-review-findings-2026-06.md");
    let runtime_15_plan = include_str!(
        "../../../../../../../docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md"
    );
    let runtime_index =
        include_str!("../../../../../../../docs/plans/zircon_runtime/runtime/index.md");
    let convention =
        include_str!("../../../../../../../docs/plans/engine-code-structure-convention.md");
    let sound_doc = include_str!("../../../../../../../docs/zircon_runtime/asset/assets/sound.md");
    let importer_doc = include_str!("../../../../../../../docs/zircon_runtime/asset/importer.md");

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
        ".unwrap()",
        ".expect(",
    ] {
        assert!(
            !sound.contains(forbidden),
            "sound WAV parsing should not keep lossy String error or panic branch `{forbidden}`"
        );
    }
    for required in [
        "SoundAssetError",
        "SoundAssetResult",
        "sound_asset_wav_parse_reports_typed_error_variants",
        "SoundAssetError::UnsupportedSpeakerMaskBits",
        "SoundAssetError::UnsupportedBitsPerSample",
        "AssetImportError::Parse(format!(",
        "\"decode wav {}: {error}\"",
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
