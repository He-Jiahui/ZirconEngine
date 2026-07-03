use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextOverflow, UiTextWrap},
};

use super::{layout_text, measure_text_size, test_style};

#[test]
fn word_wrap_keeps_non_breaking_space_group_together() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("a", &style).width + 0.1;

    let layout = layout_text(
        "a\u{00a0}b",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "a\u{00a0}b");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "NBSP is glue: the unbreakable group may overhang instead of being split by glyph fallback"
    );
}

#[test]
fn word_wrap_keeps_zwj_emoji_sequence_together() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let emoji = "👩\u{200d}💻";
    let emoji_width = measure_text_size(emoji, &style).width;
    let frame_width = (emoji_width * 0.5).max(1.0);
    assert!(emoji_width > frame_width);

    let layout = layout_text(
        emoji,
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, emoji);
    assert!(
        layout.lines[0].measured_width > frame_width,
        "ZWJ emoji sequences are glue: they may overhang but must not split"
    );
}

#[test]
fn word_wrap_keeps_variation_selector_sequence_together() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let sequence = "✈\u{fe0f}";
    let sequence_width = measure_text_size(sequence, &style).width;
    let frame_width = (sequence_width * 0.5).max(1.0);
    assert!(sequence_width > frame_width);

    let layout = layout_text(
        sequence,
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, sequence);
    assert!(
        layout.lines[0].measured_width > frame_width,
        "variation selector sequences are glue: they may overhang but must not split"
    );
}

#[test]
fn word_wrap_keeps_additional_glue_sequences_together() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);

    for sequence in ["a\u{2011}b", "a\u{202f}b", "a\u{2060}b", "a\u{feff}b"] {
        let frame_width = measure_text_size("a", &style).width + 0.1;

        let layout = layout_text(
            sequence,
            &style,
            UiFrame::new(0.0, 0.0, frame_width, 48.0),
            None,
        );

        assert_eq!(layout.lines.len(), 1, "{sequence:?} must stay unbroken");
        assert_eq!(layout.lines[0].text, sequence);
        assert!(
            layout.lines[0].measured_width > frame_width,
            "{sequence:?} is glue: it may overhang but must not split"
        );
    }
}
