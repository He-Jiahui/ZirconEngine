use zircon_runtime_interface::ui::surface::{UiResolvedStyle, UiTextDirection, UiTextRange};

use super::shape_horizontal_line;

#[test]
fn text_shape_clusters_map_source_ranges_monotonic() {
    let style = test_style();
    let source = "xxa\u{0304}\u{0301}b";
    let line_text = &source[2..];

    let shaped = shape_horizontal_line(
        line_text,
        &style,
        UiTextDirection::LeftToRight,
        UiTextRange {
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
        UiTextDirection::RightToLeft,
        UiTextRange {
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
        UiTextDirection::LeftToRight,
        UiTextRange { start: 0, end: 2 },
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
        UiTextDirection::LeftToRight,
        UiTextRange { start: 10, end: 12 },
    );

    let line = shaped.lines.first().expect("shaped line");
    let advance_sum = line.glyphs.iter().map(|glyph| glyph.advance).sum::<f32>();
    assert!((advance_sum - line.measured_width).abs() < 0.1);
    assert_eq!(line.glyphs.first().expect("glyph").source_range.start, 10);
    assert_eq!(line.glyphs.last().expect("glyph").source_range.end, 12);
}

#[test]
fn text_shape_uax14_soft_break_flags_follow_word_spaces() {
    let style = test_style();
    let text = "Hello world";

    let shaped = shape_horizontal_line(
        text,
        &style,
        UiTextDirection::LeftToRight,
        UiTextRange {
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
        UiTextDirection::LeftToRight,
        UiTextRange {
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
        UiTextDirection::LeftToRight,
        UiTextRange {
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
        UiTextDirection::LeftToRight,
        UiTextRange {
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

fn test_style() -> UiResolvedStyle {
    UiResolvedStyle {
        font_size: 10.0,
        line_height: 12.0,
        ..UiResolvedStyle::default()
    }
}
