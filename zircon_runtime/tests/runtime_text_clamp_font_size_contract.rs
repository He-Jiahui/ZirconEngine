#![cfg(feature = "ui")]

use zircon_runtime::ui::surface::layout_text;
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiResolvedStyle, UiTextOverflow, UiTextWrap},
};

#[test]
fn public_runtime_text_layout_clamps_font_size_to_fit_width() {
    let style = UiResolvedStyle {
        font_size: 24.0,
        line_height: 30.0,
        wrap: UiTextWrap::None,
        text_overflow: UiTextOverflow::ClampFontSize {
            min_px: 8.0,
            max_px: 18.0,
        },
        ..UiResolvedStyle::default()
    };

    let layout = layout_text(
        "Clamp",
        &style,
        UiFrame::new(0.0, 0.0, 32.0, style.line_height),
        None,
    );

    assert_eq!(
        layout.overflow,
        UiTextOverflow::ClampFontSize {
            min_px: 8.0,
            max_px: 18.0,
        }
    );
    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "Clamp");
    assert!(!layout.lines[0].ellipsized);
    assert!(layout.font_size >= 8.0);
    assert!(layout.font_size <= 18.0);
    assert!(layout.font_size < style.font_size);
    assert!(layout.line_height < style.line_height);
    assert!(layout.measured_width <= 32.5);
}
