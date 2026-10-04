use std::collections::HashSet;

use swash::FontRef;

use super::{
    font_identity, GlyphRasterService, RuntimeGlyphRasterFace, GLYPH_RASTER_PROFILE_COUNTER_NAMES,
};
use crate::core::framework::text::{
    TextFontCollectionHandle, TextFontFaceHandle, TextGlyphBitmapFormat, TextGlyphRasterError,
    TextGlyphRasterHinting, TextGlyphRasterMode, TextGlyphRasterRequest,
};

const TEST_FONT_BYTES: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/fonts/FiraSans-Regular.ttf"
));
const TEST_COLLECTION: TextFontCollectionHandle = TextFontCollectionHandle::new(37);
const TEST_FACE: TextFontFaceHandle = TextFontFaceHandle::new(TEST_COLLECTION, 5, 17);
const TEST_GENERATION: u64 = 17;
const TEST_SOURCE_IDENTITY: [u8; 16] = [0x5a; 16];

fn test_face() -> RuntimeGlyphRasterFace<'static> {
    RuntimeGlyphRasterFace {
        font_collection: TEST_COLLECTION,
        font_face: TEST_FACE,
        font_instance: None,
        font_generation: TEST_GENERATION,
        source_identity: TEST_SOURCE_IDENTITY,
        bytes: TEST_FONT_BYTES,
        collection_index: 0,
        variations: None,
    }
}

#[test]
fn glyph_raster_profile_uses_only_fixed_names() {
    let unique = GLYPH_RASTER_PROFILE_COUNTER_NAMES
        .into_iter()
        .collect::<HashSet<_>>();
    assert_eq!(unique.len(), GLYPH_RASTER_PROFILE_COUNTER_NAMES.len());
    assert!(unique
        .iter()
        .all(|name| name.starts_with("text_glyph_raster_")));
}

#[test]
fn text_runtime_raster_authority_font_identity_preserves_exact_source_bits() {
    let source_identity = [
        0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77, 0x88, 0x99, 0xaa, 0xbb, 0xcc, 0xdd, 0xee,
        0xff,
    ];

    assert_eq!(
        font_identity(source_identity),
        [0x7766_5544_3322_1100, 0xffee_ddcc_bbaa_9988]
    );
    let mut different_source = source_identity;
    different_source[15] ^= 1;
    assert_ne!(
        font_identity(different_source),
        font_identity(source_identity)
    );
}

#[test]
fn text_runtime_raster_authority_service_returns_exact_identity_and_real_alpha_bitmap() {
    let font = FontRef::from_index(TEST_FONT_BYTES, 0).expect("test font should parse");
    let glyph_id = u32::from(font.charmap().map('P'));
    assert_ne!(glyph_id, 0, "test glyph should be present in Fira Sans");
    let request = TextGlyphRasterRequest::new(glyph_id, 18, TextGlyphRasterMode::Outline)
        .with_hinting(TextGlyphRasterHinting::Full)
        .with_subpixel_position(20.8, 4.6);

    let receipt = GlyphRasterService::new()
        .rasterize(test_face(), request)
        .expect("real outline glyph should rasterize through the shared service");

    assert_eq!(receipt.font_collection, TEST_COLLECTION);
    assert_eq!(receipt.font_face, TEST_FACE);
    assert_eq!(receipt.font_instance, None);
    assert_eq!(receipt.font_generation, TEST_GENERATION);
    assert_eq!(receipt.source_identity, TEST_SOURCE_IDENTITY);
    assert_eq!(receipt.request, request);
    assert_eq!(receipt.request.horizontal_phase, 2);
    assert_eq!(receipt.request.vertical_phase, 2);
    assert_eq!(receipt.format, TextGlyphBitmapFormat::AlphaMask);
    assert!(receipt.size[0] > 0 && receipt.size[1] > 0);
    assert!(receipt.bearing.into_iter().all(f32::is_finite));
    assert_eq!(
        receipt.bitmap.len(),
        receipt.size[0] as usize * receipt.size[1] as usize
    );
}

#[test]
fn text_runtime_raster_authority_service_rejects_invalid_identity_before_backend_work() {
    let service = GlyphRasterService::new();

    let zero_ppem = TextGlyphRasterRequest::new(1, 0, TextGlyphRasterMode::Outline);
    assert_eq!(
        service.rasterize(test_face(), zero_ppem),
        Err(TextGlyphRasterError::InvalidPhysicalPpem)
    );

    let invalid_phase = TextGlyphRasterRequest::new(1, 18, TextGlyphRasterMode::Outline)
        .with_subpixel_phase(TextGlyphRasterRequest::HORIZONTAL_PHASE_COUNT, 0);
    assert_eq!(
        service.rasterize(test_face(), invalid_phase),
        Err(TextGlyphRasterError::InvalidSubpixelPhase)
    );

    let invalid_glyph = TextGlyphRasterRequest::new(u32::MAX, 18, TextGlyphRasterMode::Outline);
    assert_eq!(
        service.rasterize(test_face(), invalid_glyph),
        Err(TextGlyphRasterError::InvalidGlyphId)
    );
}
