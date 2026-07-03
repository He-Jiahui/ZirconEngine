use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiTextOverflow, UiTextWrap, UiTextWritingMode},
};

use super::{layout_text, measure_text_size, test_style};

#[test]
fn text_wrap_cjk_kinsoku_no_leading_punctuation() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);

    let layout = layout_text("中文。", &style, UiFrame::new(0.0, 0.0, 12.0, 48.0), None);

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "中");
    assert_eq!(layout.lines[1].text, "文。");
    assert!(
        layout.lines.iter().all(|line| !line.text.starts_with('。')),
        "CJK kinsoku must prevent forbidden punctuation from starting a wrapped line"
    );
}

#[test]
fn text_wrap_cjk_kinsoku_no_trailing_open_punctuation() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("中", &style).width + 0.1;

    let layout = layout_text(
        "中（文",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "中");
    assert_eq!(layout.lines[1].text, "（文");
    assert!(
        layout.lines.iter().all(|line| !line.text.ends_with('（')),
        "CJK kinsoku must prevent opening punctuation from ending a wrapped line"
    );
}

#[test]
fn text_wrap_cjk_kinsoku_no_leading_halfwidth_small_kana() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("中", &style).width + 0.1;

    let layout = layout_text(
        "中ｧ",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "中ｧ");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "halfwidth small kana is kinsoku glue: it may overhang but must not start a wrapped line"
    );
}

#[test]
fn text_wrap_cjk_kinsoku_no_leading_spacing_voicing_mark() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("カ", &style).width + 0.1;

    let layout = layout_text(
        "カ゛",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "カ゛");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "spacing voicing marks are kinsoku glue: they may overhang but must not start a wrapped line"
    );
}

#[test]
fn text_wrap_cjk_kinsoku_no_leading_jlreq_hyphen() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("文", &style).width + 0.1;

    let layout = layout_text(
        "文‐",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "文‐");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "JLREQ hyphens are kinsoku glue: they may overhang but must not start a wrapped line"
    );
}

#[test]
fn text_wrap_keeps_jlreq_inseparable_ellipsis_pair_together() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("…", &style).width + 0.1;

    let layout = layout_text(
        "……",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "……");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "JLREQ cl-08 inseparable pairs may overhang but must not split between matching marks"
    );
}

#[test]
fn text_wrap_cjk_kinsoku_no_leading_small_katakana_ka() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("一", &style).width + 0.1;

    let layout = layout_text(
        "一ヵ",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "一ヵ");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "small katakana ka is kinsoku glue: it may overhang but must not start a wrapped line"
    );
}

#[test]
fn text_wrap_cjk_kinsoku_no_leading_katakana_phonetic_extension_small_kana() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("一", &style).width + 0.1;

    let layout = layout_text(
        "一ㇰ",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "一ㇰ");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "katakana phonetic extension small kana is kinsoku glue: it may overhang but must not start a wrapped line"
    );
}

#[test]
fn text_wrap_cjk_kinsoku_no_leading_cjk_white_close_punctuation() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("文", &style).width + 0.1;

    let layout = layout_text(
        "文〗",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "文〗");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "CJK white closing punctuation is kinsoku glue: it may overhang but must not start a wrapped line"
    );
}

#[test]
fn text_wrap_cjk_kinsoku_no_leading_cjk_double_prime_closing_quote() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("文", &style).width + 0.1;

    let layout = layout_text(
        "文〞",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "文〞");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "CJK double-prime closing quote is kinsoku glue: it may overhang but must not start a wrapped line"
    );
}

#[test]
fn text_wrap_cjk_kinsoku_no_leading_fullwidth_white_close_parenthesis() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("文", &style).width + 0.1;

    let layout = layout_text(
        "文｠",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "文｠");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "fullwidth white close parenthesis is kinsoku glue: it may overhang but must not start a wrapped line"
    );
}

#[test]
fn text_wrap_cjk_kinsoku_no_leading_prolonged_sound_mark() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("カ", &style).width + 0.1;

    let layout = layout_text(
        "カー",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "カー");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "fullwidth prolonged sound mark is kinsoku glue: it may overhang but must not start a wrapped line"
    );
}

#[test]
fn text_wrap_cjk_kinsoku_no_leading_iteration_mark() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("時", &style).width + 0.1;

    let layout = layout_text(
        "時々",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "時々");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "iteration marks are kinsoku glue: they may overhang but must not start a wrapped line"
    );
}

#[test]
fn text_wrap_cjk_kinsoku_no_leading_vertical_iteration_mark() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("時", &style).width + 0.1;

    let layout = layout_text(
        "時〻",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "時〻");
    assert!(
        layout.lines[0].measured_width > frame_width,
        "vertical iteration marks are kinsoku glue: they may overhang but must not start a wrapped line"
    );
}

#[test]
fn text_wrap_cjk_kinsoku_no_trailing_cjk_white_open_punctuation() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("文", &style).width + 0.1;

    let layout = layout_text(
        "文〖字",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "文");
    assert_eq!(layout.lines[1].text, "〖字");
    assert!(
        layout.lines.iter().all(|line| !line.text.ends_with('〖')),
        "CJK kinsoku must prevent white opening punctuation from ending a wrapped line"
    );
}

#[test]
fn text_vertical_kinsoku_applies_to_column_break() {
    let mut style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let frame_height = measure_text_size("文", &style).width + 0.1;

    let layout = layout_text(
        "文〖字",
        &style,
        UiFrame::new(0.0, 0.0, style.line_height * 3.0, frame_height),
        None,
    );

    assert_eq!(layout.writing_mode, UiTextWritingMode::VerticalRl);
    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "文");
    assert_eq!(layout.lines[1].text, "〖字");
    assert!(layout.lines[0].frame.x > layout.lines[1].frame.x);
    assert!(
        layout.lines.iter().all(|line| !line.text.ends_with('〖')),
        "vertical CJK kinsoku must prevent white opening punctuation from ending a column"
    );
}

#[test]
fn text_wrap_cjk_kinsoku_no_trailing_fullwidth_white_open_parenthesis() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("文", &style).width + 0.1;

    let layout = layout_text(
        "文｟字",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "文");
    assert_eq!(layout.lines[1].text, "｟字");
    assert!(
        layout.lines.iter().all(|line| !line.text.ends_with('｟')),
        "CJK kinsoku must prevent fullwidth white opening parenthesis from ending a wrapped line"
    );
}

#[test]
fn text_wrap_cjk_kinsoku_no_trailing_halfwidth_open_punctuation() {
    let style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    let frame_width = measure_text_size("中", &style).width + 0.1;

    let layout = layout_text(
        "中｢文",
        &style,
        UiFrame::new(0.0, 0.0, frame_width, 48.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "中");
    assert_eq!(layout.lines[1].text, "｢文");
    assert!(
        layout.lines.iter().all(|line| !line.text.ends_with('｢')),
        "CJK kinsoku must prevent halfwidth opening punctuation from ending a wrapped line"
    );
}
