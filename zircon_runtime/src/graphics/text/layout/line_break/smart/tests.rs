use super::*;
use zircon_runtime_interface::ui::surface::UiTextRange;

fn test_chunk(text: &str, start: usize, end: usize) -> LineBreakChunk<'_> {
    LineBreakChunk {
        text: &text[start..end],
        visual_range: UiTextRange { start, end },
        source_range: UiTextRange { start, end },
        allow_glyph_fallback: true,
        break_suffix: None,
    }
}

#[test]
fn word_smart_merges_ascii_trailing_punctuation_with_previous_chunk() {
    let text = "go,next";
    let split_word = "go".len();
    let split_punctuation = "go,".len();
    let chunks = vec![
        test_chunk(text, 0, split_word),
        test_chunk(text, split_word, split_punctuation),
        test_chunk(text, split_punctuation, text.len()),
    ];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go,");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_marks_single_chunk_trailing_punctuation_unbreakable() {
    let text = "go,";
    let chunks = vec![test_chunk(text, 0, text.len())];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 1);
    assert_eq!(adjusted[0].text, text);
    assert!(!adjusted[0].allow_glyph_fallback);
}

#[test]
fn word_smart_splits_ascii_closing_quote_after_trailing_punctuation_with_word() {
    let text = "go,\"next";
    let chunks = vec![test_chunk(text, 0, text.len())];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go,\"");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_unicode_closing_quote_after_trailing_punctuation_with_word() {
    let text = "go,\u{201d}next";
    let chunks = vec![test_chunk(text, 0, text.len())];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go,\u{201d}");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_fullwidth_trailing_punctuation_with_word() {
    let text = "go，next";
    let chunks = vec![test_chunk(text, 0, text.len())];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go，");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_unicode_ellipsis_trailing_punctuation_with_word() {
    let text = "go…next";
    let chunks = vec![test_chunk(text, 0, text.len())];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go…");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_two_dot_ellipsis_trailing_punctuation_with_word() {
    let text = "go‥next";
    let chunks = vec![test_chunk(text, 0, text.len())];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go‥");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_unicode_interrobang_punctuation_with_word() {
    let text = "go\u{203d}next";
    let chunks = vec![test_chunk(text, 0, text.len())];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go\u{203d}");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_unicode_interrobang_ascii_trailing_punctuation_cluster_with_word() {
    let text = "go\u{203d}!next";
    let chunks = vec![test_chunk(text, 0, text.len())];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go\u{203d}!");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_unicode_double_punctuation_with_word() {
    for (text, protected) in [
        ("go\u{203c}next", "go\u{203c}"),
        ("go\u{2047}next", "go\u{2047}"),
        ("go\u{2048}next", "go\u{2048}"),
        ("go\u{2049}next", "go\u{2049}"),
    ] {
        let chunks = vec![test_chunk(text, 0, text.len())];

        let adjusted = apply_word_smart_rules(text, chunks);

        assert_eq!(adjusted.len(), 2, "{text}");
        assert_eq!(adjusted[0].text, protected, "{text}");
        assert!(!adjusted[0].allow_glyph_fallback, "{text}");
        assert_eq!(adjusted[1].text, "next", "{text}");
    }
}

#[test]
fn word_smart_splits_unicode_double_ascii_trailing_punctuation_cluster_with_word() {
    let text = "go\u{2049}!next";
    let chunks = vec![test_chunk(text, 0, text.len())];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go\u{2049}!");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_arabic_trailing_punctuation_with_word() {
    for (text, protected) in [
        ("go\u{060c}next", "go\u{060c}"),
        ("go\u{061b}next", "go\u{061b}"),
        ("go\u{061f}next", "go\u{061f}"),
    ] {
        let chunks = vec![test_chunk(text, 0, text.len())];

        let adjusted = apply_word_smart_rules(text, chunks);

        assert_eq!(adjusted.len(), 2, "{text}");
        assert_eq!(adjusted[0].text, protected, "{text}");
        assert!(!adjusted[0].allow_glyph_fallback, "{text}");
        assert_eq!(adjusted[1].text, "next", "{text}");
    }
}

#[test]
fn word_smart_splits_arabic_ascii_trailing_punctuation_cluster_with_word() {
    let text = "go\u{061f}!next";
    let chunks = vec![test_chunk(text, 0, text.len())];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go\u{061f}!");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_ascii_trailing_punctuation_cluster_with_word() {
    let text = "go?!next";
    let chunks = vec![test_chunk(text, 0, text.len())];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go?!");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_fullwidth_trailing_punctuation_cluster_with_word() {
    let text = "go！？next";
    let chunks = vec![test_chunk(text, 0, text.len())];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go！？");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_closing_delimiter_punctuation_cluster_with_word() {
    let text = "go，」！next";
    let chunks = vec![test_chunk(text, 0, text.len())];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go，」！");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_cjk_closing_delimiter_after_fullwidth_punctuation_with_word() {
    let text = "go，」next";
    let chunks = vec![test_chunk(text, 0, text.len())];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go，」");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_fullwidth_quote_after_trailing_punctuation_with_word() {
    let text = "go，＂next";
    let chunks = vec![test_chunk(text, 0, text.len())];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go，＂");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_leading_ascii_trailing_punctuation_chunk_before_merge() {
    let text = "go,next";
    let split_word = "go".len();
    let chunks = vec![
        test_chunk(text, 0, split_word),
        test_chunk(text, split_word, text.len()),
    ];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go,");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_leading_ascii_quote_after_trailing_punctuation_chunk_before_merge() {
    let text = "go,\"next";
    let split_word = "go".len();
    let chunks = vec![
        test_chunk(text, 0, split_word),
        test_chunk(text, split_word, text.len()),
    ];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go,\"");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_leading_unicode_quote_after_trailing_punctuation_chunk_before_merge() {
    let text = "go,\u{201d}next";
    let split_word = "go".len();
    let chunks = vec![
        test_chunk(text, 0, split_word),
        test_chunk(text, split_word, text.len()),
    ];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go,\u{201d}");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_leading_fullwidth_punctuation_chunk_before_merge() {
    let text = "go，next";
    let split_word = "go".len();
    let chunks = vec![
        test_chunk(text, 0, split_word),
        test_chunk(text, split_word, text.len()),
    ];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go，");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_splits_leading_cjk_closing_delimiter_chunk_before_merge() {
    let text = "go，」next";
    let split_word = "go".len();
    let chunks = vec![
        test_chunk(text, 0, split_word),
        test_chunk(text, split_word, text.len()),
    ];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "go，」");
    assert!(!adjusted[0].allow_glyph_fallback);
    assert_eq!(adjusted[1].text, "next");
}

#[test]
fn word_smart_marks_ascii_closing_quote_after_trailing_punctuation_unbreakable() {
    let text = "go,\"";
    let chunks = vec![test_chunk(text, 0, text.len())];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 1);
    assert_eq!(adjusted[0].text, text);
    assert!(!adjusted[0].allow_glyph_fallback);
}

#[test]
fn word_smart_marks_unicode_closing_quote_after_trailing_punctuation_unbreakable() {
    for text in ["go,\u{201d}", "go,\u{2019}"] {
        let chunks = vec![test_chunk(text, 0, text.len())];

        let adjusted = apply_word_smart_rules(text, chunks);

        assert_eq!(adjusted.len(), 1);
        assert_eq!(adjusted[0].text, text);
        assert!(!adjusted[0].allow_glyph_fallback);
    }
}

#[test]
fn word_smart_marks_fullwidth_trailing_punctuation_unbreakable() {
    for text in ["go，", "go？", "go。"] {
        let chunks = vec![test_chunk(text, 0, text.len())];

        let adjusted = apply_word_smart_rules(text, chunks);

        assert_eq!(adjusted.len(), 1);
        assert_eq!(adjusted[0].text, text);
        assert!(!adjusted[0].allow_glyph_fallback);
    }
}

#[test]
fn word_smart_marks_cjk_closing_delimiter_after_trailing_punctuation_unbreakable() {
    for text in ["go，」", "go,）", "go，＂"] {
        let chunks = vec![test_chunk(text, 0, text.len())];

        let adjusted = apply_word_smart_rules(text, chunks);

        assert_eq!(adjusted.len(), 1, "{text}");
        assert_eq!(adjusted[0].text, text, "{text}");
        assert!(!adjusted[0].allow_glyph_fallback, "{text}");
    }
}

#[test]
fn word_smart_leaves_leading_punctuation_without_previous_chunk() {
    let text = ",go";
    let comma_end = ",".len();
    let chunks = vec![
        test_chunk(text, 0, comma_end),
        test_chunk(text, comma_end, text.len()),
    ];

    let adjusted = apply_word_smart_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, ",");
    assert_eq!(adjusted[1].text, "go");
}
