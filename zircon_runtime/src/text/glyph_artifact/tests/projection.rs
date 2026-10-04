use crate::core::framework::text::{TextGlyphFlags, TextGlyphRotation};

use super::{apply_vertical_origin_offsets, TextGlyph};

fn glyph(offset_y: f32) -> TextGlyph {
    TextGlyph {
        glyph_id: 7,
        source_range: 0..1,
        visual_range: 0..1,
        advance: 4.0,
        position: [0.0, 0.0],
        offset: [0.25, offset_y],
        font_face: None,
        font_instance: None,
        rotation: TextGlyphRotation::None,
        bidi_level: 0,
        flags: TextGlyphFlags::default(),
        requires_rasterization: false,
    }
}

#[test]
fn vertical_origin_offsets_adjust_only_artifact_glyph_offsets() {
    let mut glyphs = vec![glyph(-1.0), glyph(2.0)];

    assert!(apply_vertical_origin_offsets(
        &mut glyphs,
        Some(&[0.5, -1.25])
    ));
    assert_eq!(glyphs[0].offset, [0.25, -0.5]);
    assert_eq!(glyphs[1].offset, [0.25, 0.75]);
    assert_eq!(glyphs[0].position, [0.0, 0.0]);
    assert_eq!(glyphs[1].position, [0.0, 0.0]);
}

#[test]
fn invalid_vertical_origin_sidecar_leaves_artifact_glyphs_unchanged() {
    let mut glyphs = vec![glyph(-1.0), glyph(2.0)];
    let original = glyphs.clone();

    assert!(!apply_vertical_origin_offsets(&mut glyphs, Some(&[0.5])));
    assert_eq!(glyphs, original);
    assert!(!apply_vertical_origin_offsets(
        &mut glyphs,
        Some(&[0.5, f32::NAN])
    ));
    assert_eq!(glyphs, original);
    let mut overflowing_glyphs = vec![glyph(f32::MAX), glyph(2.0)];
    let overflowing_original = overflowing_glyphs.clone();
    assert!(!apply_vertical_origin_offsets(
        &mut overflowing_glyphs,
        Some(&[f32::MAX, 0.0])
    ));
    assert_eq!(overflowing_glyphs, overflowing_original);
}
