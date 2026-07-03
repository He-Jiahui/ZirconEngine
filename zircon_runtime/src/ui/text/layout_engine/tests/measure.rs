use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextOverflow, UiTextWrap},
};

use super::{layout_text, measure_text_size, test_style};

#[test]
fn text_measurement_uses_backend_glyph_metrics() {
    let style = test_style(UiTextWrap::None, UiTextOverflow::Clip);

    let narrow = measure_text_size("iii", &style);
    let wide = measure_text_size("WWW", &style);
    let combined = measure_text_size("a\u{0301}b", &style);

    assert!(
        wide.width > narrow.width,
        "text measurement must use backend glyph metrics instead of a fixed grapheme advance"
    );
    assert!(combined.width < wide.width);
    assert_eq!(combined.height, 12.0);
}

#[test]
fn text_layout_exports_backend_grapheme_advances() {
    let style = test_style(UiTextWrap::None, UiTextOverflow::Clip);

    let layout = layout_text("Wi", &style, UiFrame::new(0.0, 0.0, 200.0, 12.0), None);

    let line = &layout.lines[0];
    assert_eq!(line.glyph_advances.len(), 2);
    assert!(
        (line.glyph_advances.iter().sum::<f32>() - line.measured_width).abs() < 0.1,
        "resolved text line must export the same backend advances used for its measured width"
    );
    assert!(
        (line.glyph_advances[0] - line.glyph_advances[1]).abs() > 0.1,
        "per-grapheme advances must preserve backend width variation"
    );
}
