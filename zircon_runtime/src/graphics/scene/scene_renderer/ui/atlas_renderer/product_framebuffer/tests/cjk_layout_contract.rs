use std::path::Path;

use crate::text::font::FontDatabase;

use super::native_layout::{NATIVE_LAYOUT_CJK_FAMILY, NATIVE_LAYOUT_TEXT};

#[test]
fn native_bitmap_layout_product_text_uses_checked_in_cjk_face() {
    assert!(
        NATIVE_LAYOUT_TEXT
            .chars()
            .any(|character| ('\u{4e00}'..='\u{9fff}').contains(&character)),
        "the native bitmap product scene must exercise embedded CJK text"
    );

    let font_source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("assets")
        .join("fonts")
        .join("ZirconDefaultComposite-subset.ttc");
    let mut font_database = FontDatabase::default();
    let face_id = font_database
        .register_font_file(&font_source, Some(NATIVE_LAYOUT_CJK_FAMILY), 1)
        .expect("register checked-in CJK product proof face");
    assert_eq!(
        font_database
            .face_family_name(face_id)
            .as_ref()
            .map(|family| family.as_str()),
        Some(NATIVE_LAYOUT_CJK_FAMILY)
    );
    let face_bytes = font_database
        .face_bytes(face_id)
        .expect("checked-in CJK product proof bytes");
    let face = ttf_parser::Face::parse(face_bytes.as_ref(), 1)
        .expect("parse checked-in CJK product proof face");
    assert!(
        NATIVE_LAYOUT_TEXT
            .chars()
            .filter(|character| ('\u{4e00}'..='\u{9fff}').contains(character))
            .all(|character| face.glyph_index(character).is_some()),
        "each CJK code point in the product scene must be drawable by the embedded proof face"
    );
}

#[test]
fn native_bitmap_layout_product_proof_enters_the_canonical_ui_glyph_pipeline() {
    let source = include_str!("native_layout.rs");

    for required in [
        "shared_font_database_snapshot",
        "layout_text",
        "UiRenderExtract",
        "native_text_batches_for_product_proof",
        "native_bitmap_atlas_glyph_runs",
        "canonical_native_layout",
    ] {
        assert!(
            source.contains(required),
            "native CJK product proof must retain `{required}`"
        );
    }
    for forbidden in [
        "TextArea",
        "TextRenderer",
        "TextAtlas",
        "layout_runs()",
        "NativeBitmapAtlasTextArea",
        "ScreenSpaceUiTextBatch {",
        concat!("refresh_screen_space_", "text_batch_glyphs"),
        "canonical_native_layout_batch",
    ] {
        assert!(
            !source.contains(forbidden),
            "native CJK product proof must not restore `{forbidden}`"
        );
    }
}
