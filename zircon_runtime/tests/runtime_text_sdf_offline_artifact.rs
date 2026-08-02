#![cfg(feature = "font-sdf-build-tool")]

use zircon_runtime::text::font_sdf_build_tool::{
    bake_font_sdf_artifact, inspect_font_sdf_artifact, FontSdfBakeMode, FontSdfBakeRequest,
    FontSdfGlyphSelection,
};

const ASSET_GUID: &str = "12345678-90ab-4cde-8f01-234567890abc";

#[test]
fn text_sdf_offline_build_is_deterministic_and_decodable_for_every_mode() {
    let font = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf"),
    )
    .expect("read FiraSans fixture");

    for mode in [
        FontSdfBakeMode::Sdf,
        FontSdfBakeMode::Msdf,
        FontSdfBakeMode::Mtsdf,
    ] {
        let request = fixture_request(mode);
        let first = bake_font_sdf_artifact(&font, &request).expect("first bake");
        let second = bake_font_sdf_artifact(&font, &request).expect("second bake");

        assert_eq!(first.bytes(), second.bytes());
        assert_eq!(first.report(), second.report());
        assert_eq!(first.report().source_context_count, 1);
        assert_eq!(first.report().source_hash_count, 1);
        assert_eq!(first.report().face_parse_count, 1);
        assert_eq!(first.report().generation_batch_count, 1);
        assert_eq!(first.report().generation_requested_glyph_count, 3);
        assert_eq!(first.report().generation_unique_glyph_count, 3);
        assert_eq!(first.report().generation_duplicate_glyph_count, 0);
        assert!(first.report().generation_worker_count >= 1);
        let inspection = inspect_font_sdf_artifact(first.bytes()).expect("decode artifact");
        assert_eq!(inspection.asset_guid, ASSET_GUID);
        assert_eq!(inspection.mode, mode);
        assert_eq!(inspection.glyph_count, first.report().generated_glyph_count);
        assert_eq!(inspection.page_count, first.report().page_count);
    }
}

#[test]
fn text_sdf_offline_inspection_rejects_corrupt_checksum() {
    let font = std::fs::read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/fonts/FiraSans-Regular.ttf"),
    )
    .expect("read FiraSans fixture");
    let artifact = bake_font_sdf_artifact(&font, &fixture_request(FontSdfBakeMode::Msdf))
        .expect("bake artifact");
    let mut corrupt = artifact.into_bytes();
    *corrupt.last_mut().expect("artifact payload") ^= 0x5a;

    assert!(inspect_font_sdf_artifact(&corrupt).is_err());
}

fn fixture_request(mode: FontSdfBakeMode) -> FontSdfBakeRequest {
    FontSdfBakeRequest {
        asset_guid: ASSET_GUID.to_string(),
        face_index: 0,
        variation_hash: *blake3::hash(&[]).as_bytes(),
        mode,
        page_size: 256,
        bake_em_px: 48,
        spread_px_milli: 8_000,
        selection: FontSdfGlyphSelection::Codepoints(vec!['A' as u32, 'M' as u32, 'g' as u32]),
    }
}
