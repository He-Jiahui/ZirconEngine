use crate::ui::text::{
    layout_text, measure_text_size,
    shaper::{UiHeuristicTextShaper, UiTextShapeRequest, UiTextShaper},
};
use zircon_runtime_interface::ui::{
    layout::{UiFrame, UiSize},
    surface::{UiResolvedStyle, UiTextOverflow, UiTextWrap},
};

#[test]
fn heuristic_text_shaper_matches_public_layout_entrypoint() {
    let style = test_style(UiTextWrap::Glyph, UiTextOverflow::Ellipsis);
    let frame = UiFrame::new(0.0, 0.0, 10.0, 12.0);
    let request = UiTextShapeRequest::new("a\u{0301}bc", &style, frame, None);

    let shaper_layout = UiHeuristicTextShaper.shape_text(&request);
    let public_layout = layout_text("a\u{0301}bc", &style, frame, None);

    assert_eq!(shaper_layout, public_layout);
    assert!(shaper_layout.lines[0].ellipsized);
    assert_eq!(shaper_layout.lines[0].text, "a\u{0301}…");
}

#[test]
fn heuristic_text_shaper_matches_public_measurement_entrypoint() {
    let style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    let shaper_size = UiHeuristicTextShaper.measure_text("a\u{0301}b", &style);

    assert_eq!(shaper_size, measure_text_size("a\u{0301}b", &style));
    assert_eq!(shaper_size, UiSize::new(10.0, 12.0));
}

fn test_style(wrap: UiTextWrap, overflow: UiTextOverflow) -> UiResolvedStyle {
    UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        wrap,
        text_overflow: overflow,
        ..UiResolvedStyle::default()
    }
}
