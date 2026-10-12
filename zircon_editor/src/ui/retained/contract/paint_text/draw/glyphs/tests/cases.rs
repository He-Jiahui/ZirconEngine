use super::*;

fn glyph(origin_x: f32, baseline_y: f32) -> RuntimeTextGlyph {
    RuntimeTextGlyph {
        glyph_id: 42,
        physical_ppem: 16,
        origin_x,
        baseline_y,
        raster_face_index: 0,
    }
}

#[test]
fn editor_request_uses_runtime_three_by_four_phase_identity() {
    let request = glyph_raster_request(
        &glyph(20.8, 40.6),
        UiTextRunPaintStyle::default(),
        HostTextSmoothing::Grayscale,
    );
    assert_eq!(request.horizontal_phase, 2);
    assert_eq!(request.vertical_phase, 2);
    assert_eq!(request.glyph_id, 42);
    assert_eq!(request.physical_ppem, 16);
    assert_eq!(request.mode, TextGlyphRasterMode::ColorPreferred);
    assert_eq!(request.hinting, TextGlyphRasterHinting::Full);
}

#[test]
fn editor_request_routes_smoothing_and_oblique_to_runtime() {
    let request = glyph_raster_request(
        &glyph(1.0, 2.0),
        UiTextRunPaintStyle {
            emphasis: true,
            ..UiTextRunPaintStyle::default()
        },
        HostTextSmoothing::Subpixel,
    );
    assert_eq!(request.smoothing, TextGlyphRasterSmoothing::Subpixel);
    assert!(request.synthetic.oblique);
}

#[test]
fn bitmap_origin_uses_floor_plus_runtime_bearing() {
    assert_eq!(bitmap_left(20.8, -1.0), 19);
    assert_eq!(bitmap_top(40.6, 12.0), 28);
    assert_eq!(bitmap_left(-0.1, 1.0), 0);
}
