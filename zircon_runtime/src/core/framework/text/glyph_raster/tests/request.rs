use super::TextGlyphRasterRequest;
use crate::core::framework::text::TextGlyphRasterMode;

#[test]
fn raster_request_quantizes_positions_into_the_shared_three_by_four_phase_grid() {
    assert_eq!(
        TextGlyphRasterRequest::horizontal_phase_for_position(20.1),
        0
    );
    assert_eq!(
        TextGlyphRasterRequest::horizontal_phase_for_position(20.45),
        1
    );
    assert_eq!(
        TextGlyphRasterRequest::horizontal_phase_for_position(20.8),
        2
    );
    assert_eq!(
        TextGlyphRasterRequest::horizontal_phase_for_position(-0.1),
        2
    );
    assert_eq!(
        TextGlyphRasterRequest::horizontal_phase_for_position(f32::NAN),
        0
    );

    assert_eq!(TextGlyphRasterRequest::vertical_phase_for_position(4.1), 0);
    assert_eq!(TextGlyphRasterRequest::vertical_phase_for_position(4.3), 1);
    assert_eq!(TextGlyphRasterRequest::vertical_phase_for_position(4.6), 2);
    assert_eq!(TextGlyphRasterRequest::vertical_phase_for_position(4.9), 3);
    assert_eq!(
        TextGlyphRasterRequest::vertical_phase_for_position(f32::INFINITY),
        0
    );
}

#[test]
fn raster_request_position_builder_uses_the_shared_phase_identity() {
    let request = TextGlyphRasterRequest::new(7, 16, TextGlyphRasterMode::Outline)
        .with_subpixel_position(20.8, 4.6);

    assert_eq!(request.horizontal_phase, 2);
    assert_eq!(request.vertical_phase, 2);
}
