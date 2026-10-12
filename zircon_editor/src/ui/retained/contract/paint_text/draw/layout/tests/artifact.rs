use zircon_runtime::core::framework::text::{TextGlyph, TextGlyphFlags, TextGlyphRotation};

use super::{artifact_glyph_geometry, physical_ppem};

#[test]
fn physical_ppem_uses_the_runtime_integer_pixel_bucket() {
    assert_eq!(physical_ppem(12.49), Some(12));
    assert_eq!(physical_ppem(12.50), Some(13));
    assert_eq!(physical_ppem(0.25), Some(1));
    assert_eq!(physical_ppem(f32::NAN), None);
}

#[test]
fn horizontal_artifact_geometry_uses_line_baseline_plus_offset_only() {
    let glyph = TextGlyph {
        glyph_id: 7,
        source_range: 0..1,
        visual_range: 0..1,
        advance: 8.0,
        position: [3.0, 99.0],
        offset: [0.25, -1.5],
        font_face: None,
        font_instance: None,
        rotation: TextGlyphRotation::None,
        bidi_level: 0,
        flags: TextGlyphFlags::default(),
        requires_rasterization: true,
    };

    let positioned =
        artifact_glyph_geometry(&glyph, 10.0, 20.0, 16, 2).expect("horizontal artifact glyph");
    assert_eq!(positioned.origin_x, 13.25);
    assert_eq!(positioned.baseline_y, 18.5);
}
