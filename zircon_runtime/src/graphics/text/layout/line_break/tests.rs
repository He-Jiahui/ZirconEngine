use super::{line_break_chunks, word_smart_line_break_chunks};
use zircon_runtime_interface::ui::surface::UiResolvedStyle;

#[test]
fn line_break_chunks_keep_cjk_open_punctuation_with_following_text() {
    let chunks = line_break_chunks("中（文", &UiResolvedStyle::default());
    let texts: Vec<_> = chunks.iter().map(|chunk| chunk.text).collect();

    assert_eq!(texts, vec!["中", "（文"]);
    assert!(!chunks[1].allow_glyph_fallback);
}

#[test]
fn line_break_chunks_keep_zwj_emoji_sequences_unbreakable() {
    let text = "a👩\u{200d}💻b";
    let chunks = line_break_chunks(text, &UiResolvedStyle::default());
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
    let chunks = line_break_chunks(text, &UiResolvedStyle::default());
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
        let chunks = line_break_chunks(text, &UiResolvedStyle::default());
        let glue_chunk = chunks
            .iter()
            .find(|chunk| !chunk.allow_glyph_fallback)
            .expect("glue chunk");

        assert_eq!(glue_chunk.text, text);
    }
}

#[test]
fn word_smart_line_break_chunks_keep_ascii_trailing_punctuation_with_word() {
    let chunks = word_smart_line_break_chunks("go,next", &UiResolvedStyle::default());
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
    let chunks = word_smart_line_break_chunks("go,\"next", &UiResolvedStyle::default());
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
    let chunks = word_smart_line_break_chunks("go,\u{201d}next", &UiResolvedStyle::default());
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
    let chunks = word_smart_line_break_chunks("go，next", &UiResolvedStyle::default());
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
    let chunks = word_smart_line_break_chunks("go…next", &UiResolvedStyle::default());
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
    let chunks = word_smart_line_break_chunks("go\u{2049}next", &UiResolvedStyle::default());
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
    let chunks = word_smart_line_break_chunks("go\u{203d}next", &UiResolvedStyle::default());
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
    let chunks = word_smart_line_break_chunks("go\u{061f}next", &UiResolvedStyle::default());
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
    let chunks = word_smart_line_break_chunks("go，」next", &UiResolvedStyle::default());
    let first_chunk = chunks
        .first()
        .expect("word smart should produce a first chunk");

    assert_eq!(first_chunk.text, "go，」");
    assert!(
        !first_chunk.allow_glyph_fallback,
        "word-smart fullwidth punctuation plus CJK closing delimiter may overhang together"
    );
}

#[test]
fn word_smart_line_break_chunks_stop_punctuation_cluster_before_next_word() {
    let chunks = word_smart_line_break_chunks("go?!next", &UiResolvedStyle::default());
    let texts: Vec<_> = chunks.iter().map(|chunk| chunk.text).collect();

    assert_eq!(texts, vec!["go?!", "next"]);
    assert!(
        !chunks[0].allow_glyph_fallback,
        "word-smart punctuation clusters may overhang but must not absorb the next word"
    );
}
