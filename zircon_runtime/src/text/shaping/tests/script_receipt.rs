use crate::core::framework::text::TextDirection;
use crate::text::TextRange;

use super::{shape_horizontal_range, test_style};

#[test]
fn text_script_segmentation_arabic_latin_runs() {
    let style = test_style();
    let text = "abc مرحبا";

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
    assert!(glyphs.iter().any(|glyph| {
        glyph.source_range.start >= 1
            && glyph.source_range.end <= text.len() - 1
            && glyph.script.iso15924 == "Zsye"
    }));
}
