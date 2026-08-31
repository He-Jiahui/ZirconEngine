use crate::core::framework::text::TextDirection;
use crate::text::{LineBreakTailoringProfile, ShapedGlyphLineBreakOpportunity, TextRange};

use super::{shape_horizontal_range, test_style};

#[test]
fn text_shape_hard_lines_preserve_separator_coverage_as_virtual_glyphs() {
    let style = test_style();
    let text = "a\r\nb\u{2028}c";
    let shaped = shape_horizontal_range(
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
            .map(|line| shaped.hard_line_text(line))
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
    assert!(virtual_breaks.iter().all(|glyph| {
        glyph.cluster_flags.mandatory_break
            && glyph.advance == 0.0
            && glyph.cluster_flags.line_break.profile == LineBreakTailoringProfile::UnicodeDefault
            && glyph.cluster_flags.line_break.opportunity
                == ShapedGlyphLineBreakOpportunity::MandatoryControl
    }));
}

#[test]
fn text_shape_uax14_soft_break_flags_follow_word_spaces() {
    let style = test_style();
    let text = "Hello world";

    let shaped = shape_horizontal_range(
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
                && glyph.cluster_flags.soft_break
                && glyph.cluster_flags.line_break.profile
                    == LineBreakTailoringProfile::UnicodeDefault
                && glyph.cluster_flags.line_break.opportunity
                    == ShapedGlyphLineBreakOpportunity::ProviderAllowed),
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

    let shaped = shape_horizontal_range(
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
            glyph.source_range.end == first_char_end
                && glyph.cluster_flags.soft_break
                && glyph.cluster_flags.line_break.profile
                    == LineBreakTailoringProfile::UnicodeDefault
                && glyph.cluster_flags.line_break.opportunity
                    == ShapedGlyphLineBreakOpportunity::ProviderAllowed
        }),
        "UAX#14 should expose a CJK ideographic break opportunity"
    );
}
