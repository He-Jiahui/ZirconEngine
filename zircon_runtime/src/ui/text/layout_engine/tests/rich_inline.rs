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
        assert_eq!(
            line.glyph_advances.len(),
            line.text.graphemes(true).count(),
            "{overflow:?}: text={:?}, runs={:?}, advances={:?}",
            line.text,
            line.runs
                .iter()
                .map(|run| run.text.as_str())
                .collect::<Vec<_>>(),
            line.glyph_advances
        );
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
    assert!(
        line.runs
            .iter()
            .any(|run| run.text == "ב" && run.direction == UiTextDirection::RightToLeft)
    );
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
    assert!(
        line.runs
            .iter()
            .any(|run| run.text == "\u{fffc}" && run.source_range.start < run.source_range.end)
    );
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
fn bbcode_inline_image_vertical_rl_composes_first_column_indent_and_continuation_height() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCode;
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let frame = UiFrame::new(0.0, 0.0, 48.0, 36.0);

    let layout = layout_text(
        "[p indent=18]甲[img=res://icons/star.png]乙丙[/p]",
        &style,
        frame,
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    assert_eq!(layout.lines[0].text, "甲");
    assert_eq!(layout.lines[1].text, "\u{fffc}乙丙");
    assert!((layout.lines[0].frame.y - (frame.y + 18.0)).abs() < 0.01);
    assert!((layout.lines[1].frame.y - frame.y).abs() < 0.01);
    assert!(layout.lines[0].frame.x > layout.lines[1].frame.x);
    assert!((layout.lines[1].glyph_advances[0] - 16.0).abs() < 0.01);
    assert!(
        layout.lines[1]
            .runs
            .iter()
            .any(|run| run.text == "\u{fffc}")
    );
    assert!(layout.lines[1].measured_width <= frame.height + 0.01);
}

#[test]
fn bbcode_inline_image_vertical_rl_composes_center_and_right_paragraph_alignment() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCode;
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let frame = UiFrame::new(0.0, 10.0, 48.0, 80.0);

    let layout = layout_text(
        "[p align=center]甲[img=res://icons/star.png]乙[/p]\n[p align=right]丙[img=res://icons/star.png]丁[/p]",
        &style,
        frame,
        None,
    );

    assert_eq!(layout.lines.len(), 2);
    let centered = &layout.lines[0];
    let ended = &layout.lines[1];
    let centered_top_gap = centered.frame.y - frame.y;
    let centered_bottom_gap = frame.bottom() - (centered.frame.y + centered.measured_width);
    assert!(centered.text.contains('\u{fffc}'));
    assert!((centered_top_gap - centered_bottom_gap).abs() < 0.01);
    assert!(centered_top_gap > 0.0);
    assert!(ended.text.contains('\u{fffc}'));
    assert!((ended.frame.y + ended.measured_width - frame.bottom()).abs() < 0.01);
}

#[test]
fn bbcode_inline_image_vertical_rl_after_empty_paragraph_uses_its_own_alignment() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Clip);
    style.rich_text_format = UiRichTextFormat::BbCode;
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let frame = UiFrame::new(0.0, 10.0, 48.0, 80.0);

    let layout = layout_text(
        "\n[p align=right]甲[img=res://icons/star.png]乙[/p]",
        &style,
        frame,
        None,
    );

    let ended = layout
        .lines
        .iter()
        .find(|line| line.text.contains('\u{fffc}'))
        .expect("rich-inline paragraph after an empty paragraph");
    assert!((ended.frame.y + ended.measured_width - frame.bottom()).abs() < 0.01);
}

#[test]
fn bbcode_inline_image_vertical_rl_word_modes_fallback_against_paragraph_heights() {
    for wrap in [UiTextWrap::Word, UiTextWrap::WordSmart] {
        let mut style = test_style(wrap, UiTextOverflow::Clip);
        style.rich_text_format = UiRichTextFormat::BbCode;
        style.text_writing_mode = UiTextWritingMode::VerticalRl;
        let frame = UiFrame::new(0.0, 10.0, 160.0, 36.0);

        let layout = layout_text(
            "[p indent=18]abcdefgh[img=res://icons/star.png]ij[/p]",
            &style,
            frame,
            None,
        );

        assert!(layout.lines.len() >= 3, "{wrap:?}");
        assert!(layout.lines[0].frame.y >= frame.y + 18.0 - 0.01, "{wrap:?}");
        assert_eq!(layout.lines[0].text, "abc", "{wrap:?}");
        assert!(
            layout.lines[0].measured_width <= frame.height - 18.0 + 0.01,
            "{wrap:?}"
        );
        assert!(
            layout
                .lines
                .iter()
                .skip(1)
                .any(|line| (line.frame.y - frame.y).abs() < 0.01),
            "{wrap:?}"
        );
        assert!(
            layout
                .lines
                .iter()
                .skip(1)
                .any(|line| line.text.graphemes(true).count() >= 4),
            "{wrap:?} continuation columns must use the full 36px height"
        );
        let inline_line = layout
            .lines
            .iter()
            .find(|line| line.text.contains('\u{fffc}'))
            .unwrap_or_else(|| panic!("{wrap:?} inline fallback column"));
        let inline_index = inline_line
            .text
            .graphemes(true)
            .position(|grapheme| grapheme == "\u{fffc}")
            .expect("inline grapheme");
        assert!((inline_line.glyph_advances[inline_index] - 16.0).abs() < 0.01);
    }
}

#[test]
fn bbcode_inline_image_vertical_rl_ellipsis_uses_paragraph_height_and_alignment() {
    let mut style = test_style(UiTextWrap::Glyph, UiTextOverflow::Ellipsis);
    style.rich_text_format = UiRichTextFormat::BbCode;
    style.text_writing_mode = UiTextWritingMode::VerticalRl;
    let frame = UiFrame::new(0.0, 10.0, 18.0, 60.0);

    let layout = layout_text(
        "[p align=right indent=10]甲[img=res://icons/star.png]乙丙丁戊[/p]",
        &style,
        frame,
        None,
    );

    assert_eq!(layout.lines.len(), 1);
    let line = &layout.lines[0];
    assert!(layout.overflow_clipped);
    assert!(line.ellipsized);
    assert!(line.text.contains('\u{fffc}'));
    assert!(line.text.ends_with('…'));
    assert!(line.measured_width <= frame.height - 10.0 + 0.01);
    assert!((line.frame.y + line.measured_width - frame.bottom()).abs() < 0.01);
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
