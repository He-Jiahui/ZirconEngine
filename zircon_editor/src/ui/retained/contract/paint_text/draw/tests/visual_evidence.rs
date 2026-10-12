use super::*;

#[test]
fn captures_actual_ink_and_clip_instead_of_bitmap_extent() {
    let clip = PixelRect {
        x0: 1,
        y0: 0,
        x1: 4,
        y1: 3,
    };
    assert_eq!(
        ink_bounds(
            &[0, 255, 0, 50, 0, 0],
            [3, 2],
            TextGlyphBitmapFormat::AlphaMask,
            [0, 0],
            &clip,
            255,
        ),
        (Some([0, 0, 2, 2]), Some([1, 0, 2, 1]))
    );
}

#[test]
fn alpha_rounding_does_not_report_pixels_the_painter_skips() {
    let clip = PixelRect {
        x0: 0,
        y0: 0,
        x1: 2,
        y1: 1,
    };
    assert_eq!(
        ink_bounds(
            &[1, 255],
            [2, 1],
            TextGlyphBitmapFormat::AlphaMask,
            [0, 0],
            &clip,
            128
        ),
        (Some([1, 0, 2, 1]), Some([1, 0, 2, 1]))
    );
    assert_eq!(
        ink_bounds(
            &[255, 255],
            [2, 1],
            TextGlyphBitmapFormat::AlphaMask,
            [0, 0],
            &clip,
            0
        ),
        (None, None)
    );
}

#[test]
fn a_failed_layout_cannot_erase_nonempty_source_without_a_diagnostic() {
    let scope = TextPaintEvidenceScope::begin().unwrap();
    let rect = FrameRect {
        x: 0.0,
        y: 0.0,
        width: 100.0,
        height: 32.0,
    };
    let clip = PixelRect {
        x0: 0,
        y0: 0,
        x1: 100,
        y1: 32,
    };
    let layout = PaintTextLayout {
        display_text: String::new(),
        glyphs: vec![],
        artifact_raster_faces: vec![],
        measured_lines: vec![],
    };
    drop(begin_run(
        "Missing text",
        &rect,
        &clip,
        14.0,
        20.0,
        UiTextRunPaintStyle::default(),
        &layout,
        HostTextLayoutPolicy::SingleLineEllipsis,
    ));
    assert_eq!(
        scope.finish().unwrap()["runs"][0]["errors"],
        json!(["nonempty source text produced no positioned glyphs"])
    );
}

#[test]
fn abandoned_scope_does_not_leak_into_the_next_capture() {
    let scope = TextPaintEvidenceScope::begin().unwrap();
    assert!(TextPaintEvidenceScope::begin().is_err());
    drop(scope);
    assert_eq!(
        TextPaintEvidenceScope::begin().unwrap().finish().unwrap()["runs"],
        json!([])
    );
}
