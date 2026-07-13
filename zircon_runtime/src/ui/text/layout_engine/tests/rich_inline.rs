use unicode_segmentation::UnicodeSegmentation;
use zircon_runtime_interface::ui::{
    layout::UiFrame,
    surface::{UiRichTextFormat, UiTextDirection, UiTextOverflow, UiTextWrap, UiTextWritingMode},
};

use super::{layout_text, test_style};

#[test]
fn html_inline_image_metrics_reach_resolved_ui_layout() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::Html;

    let layout = layout_text(
        "before<img src=\"res://icons/star.png\" width=\"16\" height=\"24\">after",
        &style,
        UiFrame::new(0.0, 0.0, 300.0, 60.0),
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert_eq!(layout.lines[0].text, "before\u{fffc}after");
    assert!(layout.line_height >= 24.0);
    assert!(layout.measured_height >= 24.0);
    assert!(layout.lines[0].baseline >= 24.0);
    assert_eq!(layout.lines[0].glyph_advances.len(), 12);
    assert!((layout.lines[0].glyph_advances[6] - 16.0).abs() < 0.01);
    let run_texts = layout.lines[0]
        .runs
        .iter()
        .map(|run| run.text.as_str())
        .collect::<Vec<_>>();
    assert_eq!(run_texts, vec!["before", "\u{fffc}", "after"]);

    let hit = crate::ui::text::hit_test_text_layout(
        &layout,
        zircon_runtime_interface::ui::layout::UiPoint::new(
            layout.lines[0].frame.x + layout.lines[0].glyph_advances[..6].iter().sum::<f32>() + 8.0,
            layout.lines[0].frame.y + 8.0,
        ),
    );
    assert!(matches!(hit.source_offset, 6 | 9));
}

#[test]
fn html_inline_image_respects_end_ellipsis_without_placeholder_fallback() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Ellipsis);
    style.rich_text_format = UiRichTextFormat::Html;
    let markup = "a<img src=\"res://icons/star.png\" width=\"16\" height=\"24\"> trailing";
    let frame = UiFrame::new(0.0, 0.0, 34.0, 60.0);

    let layout = layout_text(markup, &style, frame, None);

    let line = &layout.lines[0];
    assert!(line.ellipsized);
    assert!(line.text.ends_with('…'));
    assert!(line.text.contains('\u{fffc}'));
    assert_eq!(
        line.runs
            .iter()
            .map(|run| run.text.as_str())
            .collect::<Vec<_>>(),
        vec!["a", "\u{fffc}", "…"]
    );
    let inline_index = line
        .text
        .graphemes(true)
        .position(|grapheme| grapheme == "\u{fffc}")
        .expect("retained inline grapheme");
    assert!((line.glyph_advances[inline_index] - 16.0).abs() < 0.01);
    assert!(line.measured_width <= frame.width + 0.01);
    assert!(layout.measured_width <= frame.width + 0.01);
    assert!(layout.overflow_clipped);
}

#[test]
fn html_inline_image_respects_start_middle_and_word_ellipsis() {
    let cases = [
        (
            UiTextOverflow::EllipsisStart,
            "leading content <img src=\"res://icons/star.png\" width=\"16\" height=\"24\">",
            UiFrame::new(0.0, 0.0, 30.0, 60.0),
        ),
        (
            UiTextOverflow::EllipsisMiddle,
            "<img src=\"res://icons/star.png\" width=\"16\" height=\"24\"> trailing content",
            UiFrame::new(0.0, 0.0, 50.0, 60.0),
        ),
        (
            UiTextOverflow::EllipsisWord,
            "a<img src=\"res://icons/star.png\" width=\"16\" height=\"24\"> tail words",
            UiFrame::new(0.0, 0.0, 34.0, 60.0),
        ),
    ];

    for (overflow, markup, frame) in cases {
        let mut style = test_style(UiTextWrap::None, overflow);
        style.rich_text_format = UiRichTextFormat::Html;
        let layout = layout_text(markup, &style, frame, None);
        let line = &layout.lines[0];

        assert!(line.ellipsized, "{overflow:?}");
        assert!(line.text.contains('…'), "{overflow:?}: {}", line.text);
        assert!(
            line.text.contains('\u{fffc}'),
            "{overflow:?}: {}",
            line.text
        );
        assert_eq!(line.glyph_advances.len(), line.text.graphemes(true).count());
        assert!(line.measured_width <= frame.width + 0.01, "{overflow:?}");
        assert!(layout.measured_width <= frame.width + 0.01, "{overflow:?}");
        match overflow {
            UiTextOverflow::EllipsisStart => assert!(line.text.starts_with('…')),
            UiTextOverflow::EllipsisMiddle => {
                assert!(!line.text.starts_with('…'));
                assert!(!line.text.ends_with('…'));
            }
            UiTextOverflow::EllipsisWord => assert!(line.text.ends_with('…')),
            _ => unreachable!(),
        }
    }
}

#[test]
fn html_inline_image_after_forced_newline_keeps_rich_line_metrics() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::Html;

    let layout = layout_text(
        "first\n<img src=\"res://icons/star.png\" width=\"16\" height=\"24\">second",
        &style,
        UiFrame::new(0.0, 0.0, 300.0, 80.0),
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "first");
    assert_eq!(layout.lines[1].text, "\u{fffc}second");
    assert!(layout.lines[1].frame.y >= layout.lines[0].frame.bottom());
    assert!(layout.lines[1].frame.height >= 24.0);
    assert!(layout.lines[1].baseline >= 24.0);
    assert_eq!(layout.lines[1].glyph_advances.len(), 7);
    assert!((layout.lines[1].glyph_advances[0] - 16.0).abs() < 0.01);
    assert_eq!(layout.lines[1].source_range.start, 6);
    assert_eq!(layout.lines[1].source_range.end, layout.source_range.end);
}

#[test]
fn html_inline_image_participates_in_glyph_soft_wrap() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::Html;

    let layout = layout_text(
        "a<img src=\"res://icons/star.png\" width=\"40\" height=\"24\">z",
        &style,
        UiFrame::new(0.0, 0.0, 30.0, 100.0),
        None,
    );

    assert_eq!(layout.lines.len(), 3);
    assert_eq!(layout.lines[0].text, "a");
    assert_eq!(layout.lines[1].text, "\u{fffc}");
    assert_eq!(layout.lines[2].text, "z");
    assert_eq!(layout.lines[1].glyph_advances, vec![40.0]);
    assert!(layout.lines[1].frame.height >= 24.0);
    assert!(layout.overflow_clipped);
}

#[test]
fn html_inline_image_participates_in_word_soft_wrap() {
    let mut style = test_style(UiTextWrap::Word, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::Html;

    let layout = layout_text(
        "a <img src=\"res://icons/star.png\" width=\"40\" height=\"24\"> z",
        &style,
        UiFrame::new(0.0, 0.0, 30.0, 100.0),
        None,
    );

    assert!(layout.lines.len() >= 3);
    let inline_line = layout
        .lines
        .iter()
        .find(|line| line.text.contains('\u{fffc}'))
        .expect("word-wrapped inline line");
    let inline_index = inline_line
        .text
        .graphemes(true)
        .position(|grapheme| grapheme == "\u{fffc}")
        .expect("inline grapheme index");
    assert!((inline_line.glyph_advances[inline_index] - 40.0).abs() < 0.01);
    assert!(inline_line.frame.height >= 24.0);
    assert!(layout.overflow_clipped);
}

#[test]
fn html_inline_image_participates_in_word_smart_soft_wrap() {
    let mut style = test_style(UiTextWrap::WordSmart, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::Html;

    let layout = layout_text(
        "prefix <img src=\"res://icons/star.png\" width=\"40\" height=\"24\"> suffix",
        &style,
        UiFrame::new(0.0, 0.0, 55.0, 120.0),
        None,
    );

    assert!(layout.lines.len() >= 3);
    let inline_line = layout
        .lines
        .iter()
        .find(|line| line.text.contains('\u{fffc}'))
        .expect("word-smart-wrapped inline line");
    let inline_index = inline_line
        .text
        .graphemes(true)
        .position(|grapheme| grapheme == "\u{fffc}")
        .expect("inline grapheme index");
    assert!((inline_line.glyph_advances[inline_index] - 40.0).abs() < 0.01);
    assert!(inline_line.frame.height >= 24.0);
}

#[test]
fn html_inline_image_follows_shared_mixed_visual_order() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::Html;
    style.text_direction = UiTextDirection::Mixed;

    let layout = layout_text(
        "A <img src=\"res://icons/star.png\" width=\"18\" height=\"24\"> אב",
        &style,
        UiFrame::new(0.0, 0.0, 180.0, 60.0),
        None,
    );

    let line = &layout.lines[0];
    assert_eq!(layout.direction, UiTextDirection::LeftToRight);
    assert_eq!(line.text, "A \u{fffc} בא");
    let inline_index = line
        .text
        .graphemes(true)
        .position(|grapheme| grapheme == "\u{fffc}")
        .expect("mixed visual inline grapheme index");
    assert!((line.glyph_advances[inline_index] - 18.0).abs() < 0.01);
    assert!(line
        .runs
        .iter()
        .any(|run| run.text == "ב" && run.direction == UiTextDirection::RightToLeft));
}

#[test]
fn html_inline_image_follows_shared_rtl_visual_order() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::Html;
    style.text_direction = UiTextDirection::RightToLeft;
    let logical = "אב <img src=\"res://icons/star.png\" width=\"18\" height=\"24\"> גד";

    let layout = layout_text(logical, &style, UiFrame::new(0.0, 0.0, 180.0, 60.0), None);

    let line = &layout.lines[0];
    assert_eq!(layout.direction, UiTextDirection::RightToLeft);
    assert_ne!(line.text, "אב \u{fffc} גד");
    let inline_index = line
        .text
        .graphemes(true)
        .position(|grapheme| grapheme == "\u{fffc}")
        .expect("rtl visual inline grapheme index");
    assert!((line.glyph_advances[inline_index] - 18.0).abs() < 0.01);
    assert_eq!(line.glyph_advances.len(), line.text.graphemes(true).count());
    assert!(line
        .runs
        .iter()
        .any(|run| run.text == "\u{fffc}" && run.source_range.start < run.source_range.end));
}

#[test]
fn html_inline_image_vertical_rl_uses_object_height_as_main_axis_advance() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::Html;
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let frame = UiFrame::new(0.0, 0.0, 48.0, 35.0);

    let layout = layout_text(
        "甲<img src=\"res://icons/star.png\" width=\"18\" height=\"24\">乙",
        &style,
        frame,
        None,
    );

    assert_eq!(layout.writing_mode, UiTextWritingMode::VerticalRl);
    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "甲\u{fffc}");
    assert_eq!(layout.lines[1].text, "乙");
    assert!(layout.lines[0].frame.x > layout.lines[1].frame.x);
    assert!(layout.lines[0].frame.width >= 18.0);
    assert!((layout.lines[0].glyph_advances[1] - 24.0).abs() < 0.01);
    assert!(layout.lines[0].measured_width <= frame.height + 0.01);
    assert!(layout.measured_height <= frame.height + 0.01);
}

#[test]
fn html_inline_image_vertical_rl_marks_clipped_columns_with_end_ellipsis() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Ellipsis);
    style.rich_text_format = UiRichTextFormat::Html;
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let frame = UiFrame::new(0.0, 0.0, 18.0, 45.0);

    let layout = layout_text(
        "甲<img src=\"res://icons/star.png\" width=\"18\" height=\"24\">乙丙丁",
        &style,
        frame,
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    assert!(layout.overflow_clipped);
    assert!(layout.lines[0].ellipsized);
    assert!(layout.lines[0].text.contains('\u{fffc}'));
    assert!(layout.lines[0].text.ends_with('…'));
    assert!(layout.lines[0].measured_width <= frame.height + 0.01);
}

#[test]
fn bbcode_paragraph_alignment_reaches_resolved_line_frames() {
    let mut style = test_style(UiTextWrap::None, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCode;

    let layout = layout_text(
        "[center]alpha\nbeta[/center][right]gamma[/right]",
        &style,
        UiFrame::new(0.0, 0.0, 200.0, 80.0),
        None,
    );

    assert_eq!(layout.lines.len(), 3);
    assert_eq!(layout.lines[0].text, "alpha");
    assert_eq!(layout.lines[1].text, "beta");
    assert_eq!(layout.lines[2].text, "gamma");
    assert!((layout.lines[0].frame.x - (200.0 - layout.lines[0].frame.width) * 0.5).abs() < 0.01);
    assert!((layout.lines[1].frame.x - (200.0 - layout.lines[1].frame.width) * 0.5).abs() < 0.01);
    assert!((layout.lines[2].frame.x - (200.0 - layout.lines[2].frame.width)).abs() < 0.01);
}
