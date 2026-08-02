use crate::core::framework::text::TextDirection;
use crate::text::{BackendShapeRequest, ShapedGlyphRotation, VerticalMode};
use crate::text::{TextRange, TextStyle};

use super::{bidi::BidiParagraph, shape_horizontal_line, shape_text};

#[test]
fn text_vertical_cjk_upright_advances_on_y() {
    let style = test_style();
    let text = "本文";
    let shaped = shape_text(BackendShapeRequest::vertical(
        text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: text.len(),
        },
        VerticalMode::Mixed,
    ));
    let line = shaped.lines.first().expect("vertical shaped line");

    assert!(line
        .glyphs
        .iter()
        .all(|glyph| glyph.rotation == ShapedGlyphRotation::None));
    assert!(line
        .glyphs
        .windows(2)
        .all(|glyphs| glyphs[1].y >= glyphs[0].y));
    assert!(line.glyphs.iter().map(|glyph| glyph.advance).sum::<f32>() > 0.0);
    assert!(shaped.measured_height > 0.0);
}

#[test]
fn text_vertical_latin_sideways_rotated() {
    let style = test_style();
    let text = "Ab";
    let shaped = shape_text(BackendShapeRequest::vertical(
        text,
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 2 },
        VerticalMode::Mixed,
    ));

    assert!(shaped.lines[0]
        .glyphs
        .iter()
        .all(|glyph| glyph.rotation == ShapedGlyphRotation::Cw90));
}

#[test]
fn text_vertical_punctuation_centered() {
    let style = test_style();
    let text = "。";
    let shaped = shape_text(BackendShapeRequest::vertical(
        text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: text.len(),
        },
        VerticalMode::Mixed,
    ));
    let glyph = shaped.lines[0].glyphs.first().expect("punctuation glyph");

    assert_eq!(glyph.rotation, ShapedGlyphRotation::None);
    assert!(glyph.offset_x.is_finite());
    assert!((glyph.x - style.font_size.max(1.0) * 0.5).abs() < 0.01);
}

#[test]
fn text_bidi_levels_preserve_ltr_rtl_isolate_boundaries() {
    let text = "abc \u{2067}אב\u{2069} xyz";
    let bidi = BidiParagraph::new(text, TextDirection::Auto);

    assert_eq!(bidi.resolved_base_direction(), TextDirection::LeftToRight);
    assert_eq!(bidi.level_for_range(TextRange { start: 0, end: 1 }) % 2, 0);
    assert_eq!(
        bidi.level_for_range(TextRange {
            start: "abc \u{2067}".len(),
            end: "abc \u{2067}א".len(),
        }) % 2,
        1
    );
    assert_eq!(
        bidi.level_for_range(TextRange {
            start: "abc \u{2067}אב\u{2069} ".len(),
            end: text.len(),
        }) % 2,
        0
    );
}

#[test]
fn text_bidi_mixed_ltr_rtl_visual_order_matches_uax9() {
    let text = "abc אבג";
    let bidi = BidiParagraph::new(text, TextDirection::Auto);
    let glyph_ranges = text
        .char_indices()
        .map(|(start, ch)| TextRange {
            start,
            end: start + ch.len_utf8(),
        })
        .collect::<Vec<_>>();

    let visual = bidi.visual_order_for_line(0..text.len(), &glyph_ranges);
    let visual_text = visual
        .into_iter()
        .map(|index| &text[glyph_ranges[index].start..glyph_ranges[index].end])
        .collect::<String>();

    assert_eq!(visual_text, "abc גבא");
}

#[test]
fn text_shape_projects_resolved_bidi_level_per_glyph() {
    let style = test_style();
    let text = "abc אבג";
    let shaped = shape_horizontal_line(
        text,
        &style,
        TextDirection::Auto,
        TextRange {
            start: 0,
            end: text.len(),
        },
    );
    let glyphs = &shaped.lines.first().expect("shaped line").glyphs;

    assert!(glyphs
        .iter()
        .any(|glyph| { glyph.source_range.start < 3 && glyph.bidi_level % 2 == 0 }));
    assert!(glyphs
        .iter()
        .any(|glyph| { glyph.source_range.start >= "abc ".len() && glyph.bidi_level % 2 == 1 }));
}

#[test]
fn text_shape_keeps_mixed_bidi_clusters_in_logical_source_order() {
    let style = test_style();
    let text = "abc אבג";
    let shaped = shape_horizontal_line(
        text,
        &style,
        TextDirection::Auto,
        TextRange {
            start: 0,
            end: text.len(),
        },
    );
    let cluster_starts = shaped.lines[0]
        .glyphs
        .iter()
        .filter(|glyph| glyph.cluster_flags.cluster_start)
        .map(|glyph| glyph.source_range.start)
        .collect::<Vec<_>>();

    assert!(
        cluster_starts
            .windows(2)
            .all(|starts| starts[0] <= starts[1]),
        "Text02 must retain logical cluster order for Text03 line-level L1/L2: {cluster_starts:?}"
    );
    assert!(shaped.lines[0]
        .glyphs
        .iter()
        .any(|glyph| glyph.bidi_level % 2 == 1));
}

#[test]
fn text_shape_clusters_map_source_ranges_monotonic() {
    let style = test_style();
    let source = "xxa\u{0304}\u{0301}b";
    let line_text = &source[2..];

    let shaped = shape_horizontal_line(
        line_text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 2,
            end: source.len(),
        },
    );

    let line = shaped.lines.first().expect("shaped line");
    assert_eq!(line.source_range.start, 2);
    assert_eq!(line.source_range.end, source.len());
    assert!(!line.glyphs.is_empty());

    let mut previous_start = 2;
    let mut cluster_start_count = 0;
    for glyph in &line.glyphs {
        assert!(glyph.source_range.start >= previous_start);
        assert!(glyph.source_range.end >= glyph.source_range.start);
        previous_start = glyph.source_range.start;
        if glyph.cluster_flags.cluster_start {
            cluster_start_count += 1;
        }
    }
    assert!(cluster_start_count >= 2);
    assert_eq!(line.glyphs.first().expect("glyph").source_range.start, 2);
    assert_eq!(
        line.glyphs.last().expect("glyph").source_range.end,
        source.len()
    );
}

#[test]
fn text_shape_flags_space_tab_and_rtl_clusters() {
    let style = test_style();
    let text = "א \t";
    let shaped = shape_horizontal_line(
        text,
        &style,
        TextDirection::RightToLeft,
        TextRange {
            start: 0,
            end: text.len(),
        },
    );

    let glyphs = &shaped.lines.first().expect("shaped line").glyphs;
    assert!(
        glyphs.iter().any(|glyph| glyph.cluster_flags.rtl),
        "RTL glyph clusters must preserve directional flags"
    );
    assert!(
        glyphs.iter().any(|glyph| glyph.cluster_flags.space),
        "space clusters must be marked"
    );
    assert!(
        glyphs.iter().any(|glyph| glyph.cluster_flags.tab),
        "tab clusters must be marked"
    );
    assert!(
        glyphs.iter().any(|glyph| glyph.cluster_flags.whitespace),
        "whitespace clusters must be marked"
    );
}

#[test]
fn text_shape_latin_widths_preserve_backend_variation() {
    let style = test_style();

    let shaped = shape_horizontal_line(
        "Wi",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 2 },
    );

    let glyphs = &shaped.lines.first().expect("shaped line").glyphs;
    assert!(glyphs.len() >= 2);
    let first = glyphs.first().expect("first glyph").advance;
    let last = glyphs.last().expect("last glyph").advance;
    assert!(
        (first - last).abs() > 0.1,
        "shaping owner must preserve backend glyph advance variation"
    );
}

#[test]
fn text_shape_ligature_source_range_covers_cluster() {
    let style = test_style();

    let shaped = shape_horizontal_line(
        "fi",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 10, end: 12 },
    );

    let line = shaped.lines.first().expect("shaped line");
    let advance_sum = line.glyphs.iter().map(|glyph| glyph.advance).sum::<f32>();
    assert!((advance_sum - line.measured_width).abs() < 0.1);
    assert_eq!(line.glyphs.first().expect("glyph").source_range.start, 10);
    assert_eq!(line.glyphs.last().expect("glyph").source_range.end, 12);
}

#[test]
fn text_shape_hard_lines_preserve_separator_coverage_as_virtual_glyphs() {
    let style = test_style();
    let text = "a\r\nb\u{2028}c";
    let shaped = shape_horizontal_line(
        text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 20,
            end: 20 + text.len(),
        },
    );

    assert_eq!(
        shaped
            .lines
            .iter()
            .map(|line| line.source_range)
            .collect::<Vec<_>>(),
        vec![
            TextRange { start: 20, end: 23 },
            TextRange { start: 23, end: 27 },
            TextRange { start: 27, end: 28 },
        ]
    );
    assert_eq!(
        shaped
            .lines
            .iter()
            .map(|line| shaped.line_text(line))
            .collect::<Vec<_>>(),
        vec![Some("a\r\n"), Some("b\u{2028}"), Some("c")]
    );
    let virtual_breaks = shaped
        .lines
        .iter()
        .flat_map(|line| &line.glyphs)
        .filter(|glyph| glyph.cluster_flags.virtual_glyph)
        .collect::<Vec<_>>();
    assert_eq!(virtual_breaks.len(), 2);
    assert_eq!(
        virtual_breaks[0].source_range,
        TextRange { start: 21, end: 23 }
    );
    assert_eq!(
        virtual_breaks[1].source_range,
        TextRange { start: 24, end: 27 }
    );
    assert!(virtual_breaks
        .iter()
        .all(|glyph| glyph.cluster_flags.mandatory_break && glyph.advance == 0.0));
}

#[test]
fn text_shape_uax14_soft_break_flags_follow_word_spaces() {
    let style = test_style();
    let text = "Hello world";

    let shaped = shape_horizontal_line(
        text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: text.len(),
        },
    );

    let glyphs = &shaped.lines.first().expect("shaped line").glyphs;
    assert!(
        glyphs
            .iter()
            .any(|glyph| glyph.source_range.end == "Hello ".len()
                && glyph.cluster_flags.soft_break),
        "UAX#14 should expose a soft break after the separating space"
    );
    assert!(
        !glyphs.iter().any(
            |glyph| glyph.source_range.end == text.len() && glyph.cluster_flags.mandatory_break
        ),
        "the synthetic end-of-text break must not be projected as content"
    );
}

#[test]
fn text_shape_uax14_soft_break_flags_follow_cjk_boundaries() {
    let style = test_style();
    let text = "中文";
    let first_char_end = text.chars().next().expect("cjk char").len_utf8();

    let shaped = shape_horizontal_line(
        text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: text.len(),
        },
    );

    let glyphs = &shaped.lines.first().expect("shaped line").glyphs;
    assert!(
        glyphs.iter().any(|glyph| {
            glyph.source_range.end == first_char_end && glyph.cluster_flags.soft_break
        }),
        "UAX#14 should expose a CJK ideographic break opportunity"
    );
}

#[test]
fn text_script_segmentation_arabic_latin_runs() {
    let style = test_style();
    let text = "abc مرحبا";

    let shaped = shape_horizontal_line(
        text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: text.len(),
        },
    );

    let glyphs = &shaped.lines.first().expect("shaped line").glyphs;
    assert!(
        glyphs
            .iter()
            .any(|glyph| glyph.source_range.start < 3 && glyph.script.iso15924 == "Latn"),
        "Latin clusters should carry a Latn script tag"
    );
    assert!(
        glyphs.iter().any(
            |glyph| glyph.source_range.start >= "abc ".len() && glyph.script.iso15924 == "Arab"
        ),
        "Arabic clusters should carry an Arab script tag"
    );
    assert!(
        glyphs.iter().any(|glyph| {
            glyph.source_range.start == 3
                && glyph.source_range.end == 4
                && glyph.script.iso15924 == "Latn"
        }),
        "common separator clusters should inherit the preceding resolved script"
    );
}

#[test]
fn text_script_segmentation_keeps_emoji_zwj_sequence_as_emoji_script() {
    let style = test_style();
    let text = "a👨‍👩‍👧b";

    let shaped = shape_horizontal_line(
        text,
        &style,
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: text.len(),
        },
    );

    let glyphs = &shaped.lines.first().expect("shaped line").glyphs;
    assert!(glyphs.iter().any(|glyph| {
        glyph.source_range.start >= 1
            && glyph.source_range.end <= text.len() - 1
            && glyph.script.iso15924 == "Zsye"
    }));
}

#[test]
fn text_shaping_projects_actual_backend_font_id() {
    let text = "Actual backend face";
    let shaped = shape_horizontal_line(
        text,
        &test_style(),
        TextDirection::LeftToRight,
        TextRange {
            start: 0,
            end: text.len(),
        },
    );

    let line = shaped.lines.first().expect("shaped line");
    assert!(!line.glyphs.is_empty());
    assert!(
        line.glyphs.iter().all(|glyph| glyph.font_id.is_some()),
        "every cosmic LayoutGlyph must project its actual backend face"
    );
}

#[test]
fn text_fallback_arabic_mark_cluster_stays_on_one_actual_backend_face() {
    let text = "نَ";
    let style = TextStyle {
        font_family: Some("Zircon Missing Primary".to_string()),
        language: Some("ar".to_string()),
        ..test_style()
    };
    let shaped = shape_horizontal_line(
        text,
        &style,
        TextDirection::RightToLeft,
        TextRange {
            start: 0,
            end: text.len(),
        },
    );
    let glyphs = &shaped.lines.first().expect("shaped line").glyphs;

    assert!(
        glyphs.len() >= 2,
        "Arabic base plus fatha should reach the actual backend as a multi-glyph cluster"
    );
    let face = glyphs[0]
        .font_id
        .expect("backend-selected fallback face for Arabic cluster");
    assert!(
        glyphs.iter().all(|glyph| glyph.font_id == Some(face)),
        "all glyphs in one grapheme cluster must retain the same actual backend face: {glyphs:?}"
    );
}

#[test]
fn text_shape_request_inherits_run_language_from_resolved_style() {
    let style = TextStyle {
        language: Some(" ja-JP ".to_string()),
        ..test_style()
    };
    let request = BackendShapeRequest::horizontal(
        "漢字",
        &style,
        TextDirection::LeftToRight,
        TextRange { start: 0, end: 6 },
    );

    assert_eq!(request.language, Some("ja-JP"));
}

fn test_style() -> TextStyle {
    TextStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..TextStyle::default()
    }
}
