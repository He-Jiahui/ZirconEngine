use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextOverflow, UiTextWrap},
};

use crate::text::TEXT_SHAPING_RUN_MAX_BYTES;

use super::{layout_text, measure_text_size, test_style};

#[test]
fn word_wrap_uses_uax14_cjk_break_opportunities() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);

    let layout = layout_text("中文", &style, UiFrame::new(0.0, 0.0, 12.0, 36.0), None);

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "中");
    assert_eq!(layout.lines[1].text, "文");
}

#[test]
fn text_wrap_long_word_falls_back_to_glyph() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("a", &style).width + 0.1;

    let layout = layout_text(
        "abcd",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 72.0),
        None,
    );

    assert_eq!(layout.lines.len(), 4);
    assert_eq!(layout.lines[0].text, "a");
    assert_eq!(layout.lines[1].text, "b");
    assert_eq!(layout.lines[2].text, "c");
    assert_eq!(layout.lines[3].text, "d");
}

#[test]
fn unwrapped_oversized_run_honors_text_shaping_cap() {
    let style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    let text = format!("{}中", "x".repeat(TEXT_SHAPING_RUN_MAX_BYTES));

    let layout = layout_text(
        &text,
        &style,
        UiFrame::new(0.0, 0.0, 120.0, style.line_height * 3.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text.len(), TEXT_SHAPING_RUN_MAX_BYTES);
    assert_eq!(layout.lines[1].text, "中");
    assert_eq!(
        layout.lines[1].source_range.start,
        TEXT_SHAPING_RUN_MAX_BYTES
    );
}
