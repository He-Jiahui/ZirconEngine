use std::sync::Arc;

use super::{line_break_chunks, line_break_chunks_with_provider, word_smart_line_break_chunks};
use crate::core::framework::text::TextDirection;
use crate::text::shaping::{DirectTextShapeRunProvider, TextShapeRunProvider};
use crate::text::{ShapedGlyphRun, TextRange, TextStyle};

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

    let chunks = line_break_chunks_with_provider(text, &TextStyle::default(), &mut provider);

    assert_eq!(provider.shape_calls, 3);
    assert_eq!(
        chunks.iter().map(|chunk| chunk.text).collect::<Vec<_>>(),
        vec!["one\n", "two\n", "three"]
    );
    assert!(chunks[0].mandatory_break);
    assert!(chunks[1].mandatory_break);
    assert!(!chunks[2].mandatory_break);
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
}

impl TextShapeRunProvider for CountingShapeRunProvider {
    fn shape_horizontal_line_with_kerning(
        &mut self,
        text: &str,
        style: &TextStyle,
        direction: TextDirection,
        source_range: TextRange,
        include_kerning: bool,
    ) -> Arc<ShapedGlyphRun> {
        self.shape_calls = self.shape_calls.saturating_add(1);
        self.direct.shape_horizontal_line_with_kerning(
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
