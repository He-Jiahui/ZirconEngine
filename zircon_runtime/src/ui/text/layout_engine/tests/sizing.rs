use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextOverflow, UiTextWrap},
};

use super::{layout_text, measure_text_size, test_style};

#[test]
fn text_shrink_to_fit_scales_within_bounds() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::ShrinkToFit);
    style.font_size = 20.0;
    style.line_height = 24.0;
    let text = "Wide runtime text";
    let natural_width = measure_text_size(text, &style).width;
    let frame_width = natural_width * 0.5;

    let layout = layout_text(
        text,
        &style,
        UiFrame::new(0.0, 0.0, frame_width, style.line_height),
        None,
    );

    assert_eq!(layout.overflow, UiTextOverflow::ShrinkToFit);
    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, text);
    assert!(!layout.lines[0].ellipsized);
    assert!(layout.font_size < style.font_size);
    assert!(layout.font_size >= 1.0);
    assert!(layout.line_height < style.line_height);
    assert!(layout.measured_width <= frame_width + 0.5);
    assert!(layout.lines[0].measured_width <= frame_width + 0.5);
}

#[test]
fn text_clamp_font_size_respects_min_max_bounds() {
    let mut style = test_style(
        UiTextWrap::None,
        UiTextOverflow::ClampFontSize {
            min_px: 8.0,
            max_px: 18.0,
        },
    );
    style.font_size = 24.0;
    style.line_height = 30.0;
    let text = "Clamp runtime text";
    let mut min_style = style.clone();
    min_style.font_size = 8.0;
    min_style.line_height = 10.0;
    let frame_width = measure_text_size(text, &min_style).width + 0.25;

    let layout = layout_text(
        text,
        &style,
        UiFrame::new(0.0, 0.0, frame_width, style.line_height),
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
    assert_eq!(layout.lines[0].text, text);
    assert!(!layout.lines[0].ellipsized);
    assert!(layout.font_size <= 18.0);
    assert!(layout.font_size >= 8.0);
    assert!(layout.line_height < style.line_height);
    assert!(layout.measured_width <= frame_width + 0.5);
}
