use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextAlign, UiTextDirection, UiTextOverflow, UiTextWrap},
};

use crate::text::layout::measured_grapheme_widths;
use crate::text::text_style;

use super::{layout_text, measure_text_size, test_style};

#[test]
fn text_justify_distributes_word_and_cjk_gaps() {
    let mut style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    style.text_align = UiTextAlign::Justify;
    let first_line = "a b 中文";
    let target_width = measure_text_size(first_line, &style).width + 24.0;

    let layout = layout_text(
        "a b 中文\ntail",
        &style,
        UiFrame::new(0.0, 0.0, target_width, 24.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, first_line);
    assert!((layout.lines[0].frame.x - 0.0).abs() < 0.01);
    assert!((layout.lines[0].frame.width - target_width).abs() < 0.1);
    assert!((layout.lines[0].measured_width - target_width).abs() < 0.1);
    assert!((layout.lines[0].glyph_advances.iter().sum::<f32>() - target_width).abs() < 0.1);

    let natural_advances = measured_grapheme_widths(first_line, &text_style(&style));
    assert_eq!(layout.lines[0].glyph_advances.len(), natural_advances.len());
    assert!(layout.lines[0].glyph_advances[1] > natural_advances[1]);
    assert!(layout.lines[0].glyph_advances[3] > natural_advances[3]);
    assert!(layout.lines[0].glyph_advances[4] > natural_advances[4]);
    assert!((layout.lines[0].glyph_advances[0] - natural_advances[0]).abs() < 0.1);

    let last_line_width = measure_text_size("tail", &style).width;
    assert!((layout.lines[1].frame.width - last_line_width).abs() < 0.1);
    assert!((layout.lines[1].measured_width - last_line_width).abs() < 0.1);
}

#[test]
fn text_justify_trims_edge_spaces_before_distributing_gaps() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_align = UiTextAlign::Justify;
    let first_line = " a b ";
    let target_width = measure_text_size(first_line, &style).width + 18.0;

    let layout = layout_text(
        " a b \ntail",
        &style,
        UiFrame::new(0.0, 0.0, target_width, 24.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, first_line);
    assert!((layout.lines[0].measured_width - target_width).abs() < 0.1);

    let natural_advances = measured_grapheme_widths(first_line, &text_style(&style));
    assert_eq!(layout.lines[0].glyph_advances.len(), natural_advances.len());
    assert!(
        (layout.lines[0].glyph_advances[0] - natural_advances[0]).abs() < 0.1,
        "leading spaces are edge whitespace and must not receive justify expansion"
    );
    assert!(
        layout.lines[0].glyph_advances[2] > natural_advances[2],
        "the interior word gap must receive the justify expansion"
    );
    assert!(
        (layout.lines[0].glyph_advances[4] - natural_advances[4]).abs() < 0.1,
        "trailing spaces are edge whitespace and must not receive justify expansion"
    );
}

#[test]
fn text_justify_distributes_arabic_kashida_advances() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.text_align = UiTextAlign::Justify;
    style.text_direction = UiTextDirection::RightToLeft;
    let first_line = "سلام";
    let target_width = measure_text_size(first_line, &style).width + 18.0;

    let layout = layout_text(
        "سلام\nذ",
        &style,
        UiFrame::new(0.0, 0.0, target_width, 24.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert!((layout.lines[0].frame.width - target_width).abs() < 0.1);
    assert!((layout.lines[0].measured_width - target_width).abs() < 0.1);
    assert!((layout.lines[0].glyph_advances.iter().sum::<f32>() - target_width).abs() < 0.1);

    let natural_advances =
        measured_grapheme_widths(layout.lines[0].text.as_str(), &text_style(&style));
    assert_eq!(layout.lines[0].glyph_advances.len(), natural_advances.len());
    assert!(
        layout.lines[0]
            .glyph_advances
            .iter()
            .zip(natural_advances.iter())
            .any(|(adjusted, natural)| *adjusted > *natural + 0.1),
        "Arabic joining opportunities should receive kashida-like justify advance"
    );
    assert!(
        layout.lines[0]
            .glyph_advances
            .iter()
            .zip(natural_advances.iter())
            .any(|(adjusted, natural)| (*adjusted - *natural).abs() < 0.1),
        "kashida justify should not scale every Arabic glyph advance"
    );

    let last_line_width = measure_text_size("ذ", &style).width;
    assert!((layout.lines[1].frame.width - last_line_width).abs() < 0.1);
}
