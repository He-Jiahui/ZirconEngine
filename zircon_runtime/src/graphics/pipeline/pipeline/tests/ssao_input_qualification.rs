use super::*;

#[test]
fn render_rect_qualification_rejects_unconsumed_partial_rects() {
    let error = validate_render_rect(
        RenderViewportRect::new(UVec2::ZERO, UVec2::new(640, 360)),
        UVec2::new(1280, 720),
    )
    .unwrap_err();

    assert!(error.contains("partial rect AO is unsupported"));
}
