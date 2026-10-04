use std::sync::Arc;

use super::{line_break_chunks, line_break_chunks_with_provider, word_smart_line_break_chunks};
use crate::core::framework::text::{TextDirection, TextLayoutError};
use crate::text::shaping::{DirectTextShapeRunProvider, TextShapeRunProvider};
use crate::text::{ShapedGlyph, ShapedGlyphRun, ShapedHardLine, TextRange, TextStyle};

#[test]
fn line_break_chunks_keep_cjk_open_punctuation_with_following_text() {
    let chunks = line_break_chunks("中（文", &TextStyle::default());
    let texts: Vec<_> = chunks.iter().map(|chunk| chunk.text).collect();

    assert_eq!(texts, vec!["中", "（文"]);
    assert!(!chunks[1].allow_glyph_fallback);
}

#[test]
fn line_break_chunks_use_absolute_offsets_and_preserve_mandatory_breaks() {
    let text = "a\u{2028}b";
    let chunks = line_break_chunks(text, &TextStyle::default());

    assert_eq!(
        chunks.iter().map(|chunk| chunk.text).collect::<Vec<_>>(),
        vec!["a\u{2028}", "b"]
    );
    assert_eq!(chunks[0].source_range.start, 0);
    assert_eq!(chunks[0].source_range.end, "a\u{2028}".len());
    assert!(chunks[0].mandatory_break);
    assert_eq!(chunks[1].source_range.start, "a\u{2028}".len());
}

#[test]
fn line_break_chunks_shape_physical_paragraphs_independently() {
    let text = "one\ntwo\nthree";
    let mut provider = CountingShapeRunProvider::default();

    let chunks = line_break_chunks_with_provider(text, &TextStyle::default(), &mut provider)
        .into_result()
        .expect("build line break chunks");

    assert_eq!(provider.shape_calls, 3);
    assert_eq!(
        provider.requested_ranges,
        vec![
            TextRange { start: 0, end: 4 },
            TextRange { start: 4, end: 8 },
            TextRange { start: 8, end: 13 },
        ]
    );
    assert_eq!(
        chunks.iter().map(|chunk| chunk.text).collect::<Vec<_>>(),
        vec!["one\n", "two\n", "three"]
    );
    assert!(chunks[0].mandatory_break);
    assert!(chunks[1].mandatory_break);
    assert!(!chunks[2].mandatory_break);
}

#[test]
fn line_break_chunks_normalize_visual_glyph_order_before_materializing_boundaries() {
    let text = "a b c";
    let shaped = Arc::new(ShapedGlyphRun {
        source_text: Arc::from(text),
        source_range: TextRange {
            start: 0,
            end: text.len(),
        },
        unicode_data_snapshot: crate::text::compiled_unicode_data_snapshot_id(),
        primary_face_id: None,
        direction: TextDirection::RightToLeft,
        orientation: crate::text::TextOrientation::Horizontal,
        vertical_mode: crate::text::VerticalMode::Mixed,
        include_kerning: true,
        measured_width: 5.0,
        measured_height: 10.0,
        horizontal_composition_receipt: None,
        horizontal_line_raw_metrics: Vec::new(),
        horizontal_glyph_metric_spans: Vec::new(),
        lines: vec![ShapedHardLine {
            line_index: 0,
            source_range: TextRange {
                start: 0,
                end: text.len(),
            },
            visual_range: TextRange {
                start: 0,
                end: text.len(),
            },
            measured_width: 5.0,
            baseline: 8.0,
            line_height: 10.0,
            glyphs: vec![break_glyph(3..4), break_glyph(1..2), plain_glyph(0..1)],
        }],
    });
    let mut provider = FixedShapeRunProvider { shaped };

    let chunks = line_break_chunks_with_provider(text, &TextStyle::default(), &mut provider)
        .into_result()
        .expect("visual-order fixture must produce logical chunks");

    assert_eq!(
        chunks.iter().map(|chunk| chunk.text).collect::<Vec<_>>(),
        vec!["a ", "b ", "c"]
    );
}

#[test]
fn line_break_chunks_fail_closed_on_non_utf8_cluster_ranges() {
    let text = "éx";
    let shaped = Arc::new(ShapedGlyphRun {
        source_text: Arc::from(text),
        source_range: TextRange {
            start: 0,
            end: text.len(),
        },
        unicode_data_snapshot: crate::text::compiled_unicode_data_snapshot_id(),
        primary_face_id: None,
        direction: TextDirection::LeftToRight,
        orientation: crate::text::TextOrientation::Horizontal,
        vertical_mode: crate::text::VerticalMode::Mixed,
        include_kerning: true,
        measured_width: 2.0,
        measured_height: 10.0,
        horizontal_composition_receipt: None,
        horizontal_line_raw_metrics: Vec::new(),
        horizontal_glyph_metric_spans: Vec::new(),
        lines: vec![ShapedHardLine {
            line_index: 0,
            source_range: TextRange {
                start: 0,
                end: text.len(),
            },
            visual_range: TextRange {
                start: 0,
                end: text.len(),
            },
            measured_width: 2.0,
            baseline: 8.0,
            line_height: 10.0,
            glyphs: vec![plain_glyph(1..2)],
        }],
    });
    let mut provider = FixedShapeRunProvider { shaped };

    let outcome = line_break_chunks_with_provider(text, &TextStyle::default(), &mut provider);

    assert_eq!(outcome.into_result(), Err(TextLayoutError::BidiInvariant));
}

#[test]
fn mandatory_breaks_stop_kinsoku_merges_across_hard_lines() {
    let text = "a\u{2028}）b";
    let chunks = line_break_chunks(text, &TextStyle::default());
    let mandatory = chunks
        .iter()
        .find(|chunk| chunk.mandatory_break)
        .expect("mandatory hard-line chunk");

    assert_eq!(mandatory.text, "a\u{2028}");
    assert_eq!(mandatory.source_range.end, "a\u{2028}".len());
}

#[test]
fn line_break_chunks_keep_zwj_emoji_sequences_unbreakable() {
    let text = "a👩\u{200d}💻b";
    let chunks = line_break_chunks(text, &TextStyle::default());
    let glue_chunk = chunks
        .iter()
        .find(|chunk| chunk.text.contains('\u{200d}'))
        .expect("ZWJ emoji chunk");

    assert!(
        !glue_chunk.allow_glyph_fallback,
        "ZWJ emoji sequences are glue and must not be split by glyph fallback"
    );
}

#[test]
fn line_break_chunks_keep_variation_selector_sequences_unbreakable() {
    let text = "a✈\u{fe0f}b";
    let chunks = line_break_chunks(text, &TextStyle::default());
    let glue_chunk = chunks
        .iter()
        .find(|chunk| chunk.text.contains('\u{fe0f}'))
        .expect("variation selector chunk");

    assert!(
        !glue_chunk.allow_glyph_fallback,
        "variation selector sequences are glue and must not be split by glyph fallback"
    );
}

#[test]
fn line_break_chunks_keep_additional_glue_sequences_unbreakable() {
    for text in ["a\u{2011}b", "a\u{202f}b", "a\u{2060}b", "a\u{feff}b"] {
        let chunks = line_break_chunks(text, &TextStyle::default());
        let glue_chunk = chunks
            .iter()
            .find(|chunk| !chunk.allow_glyph_fallback)
            .expect("glue chunk");

        assert_eq!(glue_chunk.text, text);
    }
}

#[test]
fn word_smart_line_break_chunks_keep_ascii_trailing_punctuation_with_word() {
    let chunks = word_smart_line_break_chunks("go,next", &TextStyle::default());
    let first_chunk = chunks
        .first()
        .expect("word smart should produce a first chunk");

    assert_eq!(first_chunk.text, "go,");
    assert!(
        !first_chunk.allow_glyph_fallback,
        "word-smart trailing punctuation may overhang but must not start a wrapped line"
    );
}

#[test]
fn word_smart_line_break_chunks_keep_ascii_quote_after_trailing_punctuation_with_word() {
    let chunks = word_smart_line_break_chunks("go,\"next", &TextStyle::default());
    let first_chunk = chunks
        .first()
        .expect("word smart should produce a first chunk");

    assert_eq!(first_chunk.text, "go,\"");
    assert!(
        !first_chunk.allow_glyph_fallback,
        "word-smart trailing punctuation plus closing quote may overhang together"
    );
}

#[test]
fn word_smart_line_break_chunks_keep_unicode_quote_after_trailing_punctuation_with_word() {
    let chunks = word_smart_line_break_chunks("go,\u{201d}next", &TextStyle::default());
    let first_chunk = chunks
        .first()
        .expect("word smart should produce a first chunk");

    assert_eq!(first_chunk.text, "go,\u{201d}");
    assert!(
        !first_chunk.allow_glyph_fallback,
        "word-smart trailing punctuation plus Unicode closing quote may overhang together"
    );
}

#[test]
fn word_smart_line_break_chunks_keep_fullwidth_trailing_punctuation_with_word() {
    let chunks = word_smart_line_break_chunks("go，next", &TextStyle::default());
    let first_chunk = chunks
        .first()
        .expect("word smart should produce a first chunk");

    assert_eq!(first_chunk.text, "go，");
    assert!(
        !first_chunk.allow_glyph_fallback,
        "word-smart fullwidth trailing punctuation may overhang but must not start a wrapped line"
    );
}

#[test]
fn word_smart_line_break_chunks_keep_ellipsis_trailing_punctuation_with_word() {
    let chunks = word_smart_line_break_chunks("go…next", &TextStyle::default());
    let first_chunk = chunks
        .first()
        .expect("word smart should produce a first chunk");

    assert_eq!(first_chunk.text, "go…");
    assert!(
        !first_chunk.allow_glyph_fallback,
        "word-smart ellipsis punctuation may overhang but must not start a wrapped line"
    );
}

#[test]
fn word_smart_line_break_chunks_keep_unicode_double_punctuation_with_word() {
    let chunks = word_smart_line_break_chunks("go\u{2049}next", &TextStyle::default());
    let first_chunk = chunks
        .first()
        .expect("word smart should produce a first chunk");

    assert_eq!(first_chunk.text, "go\u{2049}");
    assert!(
        !first_chunk.allow_glyph_fallback,
        "word-smart Unicode double punctuation may overhang but must not start a wrapped line"
    );
}

#[test]
fn word_smart_line_break_chunks_keep_unicode_interrobang_punctuation_with_word() {
    let chunks = word_smart_line_break_chunks("go\u{203d}next", &TextStyle::default());
    let first_chunk = chunks
        .first()
        .expect("word smart should produce a first chunk");

    assert_eq!(first_chunk.text, "go\u{203d}");
    assert!(
        !first_chunk.allow_glyph_fallback,
        "word-smart Unicode interrobang punctuation may overhang but must not start a wrapped line"
    );
}

#[test]
fn word_smart_line_break_chunks_keep_arabic_trailing_punctuation_with_word() {
    let chunks = word_smart_line_break_chunks("go\u{061f}next", &TextStyle::default());
    let first_chunk = chunks
        .first()
        .expect("word smart should produce a first chunk");

    assert_eq!(first_chunk.text, "go\u{061f}");
    assert!(
        !first_chunk.allow_glyph_fallback,
        "word-smart Arabic punctuation may overhang but must not start a wrapped line"
    );
}

#[test]
fn word_smart_line_break_chunks_keep_cjk_closing_delimiter_after_punctuation_with_word() {
    let chunks = word_smart_line_break_chunks("go，」next", &TextStyle::default());
    let first_chunk = chunks
        .first()
        .expect("word smart should produce a first chunk");

    assert_eq!(first_chunk.text, "go，」");
    assert!(
        !first_chunk.allow_glyph_fallback,
        "word-smart fullwidth punctuation plus CJK closing delimiter may overhang together"
    );
}

#[derive(Default)]
struct CountingShapeRunProvider {
    direct: DirectTextShapeRunProvider,
    shape_calls: usize,
    requested_ranges: Vec<TextRange>,
}

struct FixedShapeRunProvider {
    shaped: Arc<ShapedGlyphRun>,
}

impl TextShapeRunProvider for FixedShapeRunProvider {
    fn shape_horizontal_range_with_kerning(
        &mut self,
        _text: &str,
        _style: &TextStyle,
        _direction: TextDirection,
        _source_range: TextRange,
        _include_kerning: bool,
    ) -> crate::text::shaping::TextShapingOutcome {
        crate::text::shaping::TextShapingOutcome::Ready(Arc::clone(&self.shaped))
    }
}

fn break_glyph(range: std::ops::Range<usize>) -> ShapedGlyph {
    ShapedGlyph {
        glyph_id: 1,
        font_id: None,
        font_instance_id: None,
        source_range: TextRange {
            start: range.start,
            end: range.end,
        },
        visual_range: TextRange {
            start: range.start,
            end: range.end,
        },
        advance: 1.0,
        x: 0.0,
        y: 0.0,
        offset_x: 0.0,
        offset_y: 0.0,
        direction: TextDirection::RightToLeft,
        bidi_level: 1,
        cluster_flags: crate::text::ShapedGlyphClusterFlags {
            cluster_start: true,
            soft_break: true,
            rtl: true,
            ..crate::text::ShapedGlyphClusterFlags::default()
        },
        rotation: crate::text::ShapedGlyphRotation::None,
        script: crate::text::ShapedGlyphScript::default(),
    }
}

fn plain_glyph(range: std::ops::Range<usize>) -> ShapedGlyph {
    ShapedGlyph {
        cluster_flags: crate::text::ShapedGlyphClusterFlags {
            cluster_start: true,
            ..crate::text::ShapedGlyphClusterFlags::default()
        },
        ..break_glyph(range)
    }
}

impl TextShapeRunProvider for CountingShapeRunProvider {
    fn shape_horizontal_range_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        include_kerning: bool,
    ) -> crate::text::shaping::TextShapingOutcome {
        self.shape_calls = self.shape_calls.saturating_add(1);
        self.requested_ranges.push(source_range);
        self.direct.shape_horizontal_range_with_kerning(
            text,
            style,
            direction,
            source_range,
            include_kerning,
        )
    }
}

#[test]
fn word_smart_line_break_chunks_stop_punctuation_cluster_before_next_word() {
    let chunks = word_smart_line_break_chunks("go?!next", &TextStyle::default());
    let texts: Vec<_> = chunks.iter().map(|chunk| chunk.text).collect();

    assert_eq!(texts, vec!["go?!", "next"]);
    assert!(
        !chunks[0].allow_glyph_fallback,
        "word-smart punctuation clusters may overhang but must not absorb the next word"
    );
}
