use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextOverflow, UiTextWrap},
};

use super::{layout_text, measure_text_size, test_style};

#[test]
fn word_smart_keeps_ascii_trailing_punctuation_with_previous_word() {
    let style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    let frame_width = measure_text_size("go", &style).width + 0.1;

    let layout = layout_text(
        "go,a",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 64.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "go,");
    assert_eq!(layout.lines[1].text, "a");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "WordSmart should let trailing punctuation overhang with its word instead of starting a line"
    );
}

#[test]
fn word_smart_keeps_ascii_closing_quote_after_trailing_punctuation_with_previous_word() {
    let style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    let frame_width = measure_text_size("go,", &style).width + 0.1;

    let layout = layout_text(
        "go,\"a",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 64.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "go,\"");
    assert_eq!(layout.lines[1].text, "a");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "WordSmart should overhang a closing quote that follows protected trailing punctuation"
    );
}

#[test]
fn word_smart_keeps_unicode_closing_quote_after_trailing_punctuation_with_previous_word() {
    let style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    let frame_width = measure_text_size("go,", &style).width + 0.1;

    let layout = layout_text(
        "go,\u{201d}a",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 64.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "go,\u{201d}");
    assert_eq!(layout.lines[1].text, "a");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "WordSmart should overhang a Unicode closing quote after protected trailing punctuation"
    );
}

#[test]
fn word_smart_keeps_fullwidth_trailing_punctuation_with_previous_word() {
    let style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    let frame_width = measure_text_size("go", &style).width + 0.1;

    let layout = layout_text(
        "go，a",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 64.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "go，");
    assert_eq!(layout.lines[1].text, "a");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "WordSmart should overhang fullwidth trailing punctuation with its previous word"
    );
}

#[test]
fn word_smart_keeps_ellipsis_trailing_punctuation_with_previous_word() {
    let style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    let frame_width = measure_text_size("go", &style).width + 0.1;

    let layout = layout_text(
        "go…a",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 64.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "go…");
    assert_eq!(layout.lines[1].text, "a");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "WordSmart should overhang Unicode ellipsis punctuation with its previous word"
    );
}

#[test]
fn word_smart_keeps_unicode_double_punctuation_with_previous_word() {
    let style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    let frame_width = measure_text_size("go", &style).width + 0.1;

    let layout = layout_text(
        "go\u{2049}a",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 64.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "go\u{2049}");
    assert_eq!(layout.lines[1].text, "a");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "WordSmart should overhang Unicode double punctuation with its previous word"
    );
}

#[test]
fn word_smart_keeps_unicode_interrobang_punctuation_with_previous_word() {
    let style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    let frame_width = measure_text_size("go", &style).width + 0.1;

    let layout = layout_text(
        "go\u{203d}a",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 64.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "go\u{203d}");
    assert_eq!(layout.lines[1].text, "a");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "WordSmart should overhang Unicode interrobang punctuation with its previous word"
    );
}

#[test]
fn word_smart_keeps_arabic_trailing_punctuation_with_previous_word() {
    let style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    let frame_width = measure_text_size("go", &style).width + 0.1;

    let layout = layout_text(
        "go\u{061f}a",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 64.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "go\u{061f}");
    assert_eq!(layout.lines[1].text, "a");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "WordSmart should overhang Arabic trailing punctuation with its previous word"
    );
}

#[test]
fn word_smart_keeps_cjk_closing_delimiter_after_fullwidth_punctuation_with_previous_word() {
    let style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    let frame_width = measure_text_size("go，", &style).width + 0.1;

    let layout = layout_text(
        "go，」a",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 64.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "go，」");
    assert_eq!(layout.lines[1].text, "a");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "WordSmart should overhang CJK closing delimiters after protected punctuation"
    );
}

#[test]
fn word_smart_keeps_trailing_punctuation_cluster_without_absorbing_next_word() {
    let style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    let frame_width = measure_text_size("go?!", &style).width + 0.1;

    let layout = layout_text(
        "go?!a",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 64.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "go?!");
    assert_eq!(layout.lines[1].text, "a");
    assert!(
        layout.lines[0].measured_width <= frame_width,
        "WordSmart should keep the punctuation cluster with the word without swallowing following text"
    );
}
