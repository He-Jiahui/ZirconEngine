//! UE Canvas text metrics, using ZE's engine-owned packaged UI font.
#![cfg(feature = "ui")]

use zircon_runtime::ui::surface::{measure_text_size, shape_text_line};
use zircon_runtime_interface::ui::surface::UiResolvedStyle;

#[test]
fn unreal_canvas_multiline_width_uses_packaged_ui_font() {
    // UE.Engine.Canvas.BasicMultilineWidth requires a real engine font and
    // compares the same three individual lines to their joined measurement.
    let style = UiResolvedStyle::default();
    assert!(style.font.is_none() && style.font_family.is_none());
    let text = "Hello\nWorld\nTest";
    let shaped = shape_text_line(text, &style).expect("shape using the packaged default font");
    assert_eq!(shaped.lines.len(), 3);
    for line in &shaped.lines {
        assert!(
            line.glyphs
                .iter()
                .any(|glyph| glyph.glyph_id != 0 && glyph.font_id.is_some()),
            "each hard line needs a glyph from a real face"
        );
    }

    let widths = ["Hello", "World", "Test"].map(|line| measure_text_size(line, &style).width);
    assert!(widths.iter().all(|width| width.is_finite() && *width > 0.0));
    let expected = widths.into_iter().fold(0.0_f32, f32::max);
    let measured = measure_text_size(text, &style).width;
    assert!(
        (measured - expected).abs() <= 0.1,
        "multiline width {measured} must equal the widest line {expected}"
    );
}
