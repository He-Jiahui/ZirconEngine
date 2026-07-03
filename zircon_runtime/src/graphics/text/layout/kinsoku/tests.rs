use super::*;

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
fn merges_halfwidth_kana_forbidden_line_start_with_previous_chunk() {
    let text = "中ｧ";
    let split = "中".len();
    let chunks = vec![
        test_chunk(text, 0, split),
        test_chunk(text, split, text.len()),
    ];

    let adjusted = apply_kinsoku_start_rules(text, chunks);

    assert_eq!(adjusted.len(), 1);
    assert_eq!(adjusted[0].text, text);
    assert!(!adjusted[0].allow_glyph_fallback);
}

#[test]
fn merges_halfwidth_voicing_mark_forbidden_line_start_with_previous_chunk() {
    let text = "カﾞ";
    let split = "カ".len();
    let chunks = vec![
        test_chunk(text, 0, split),
        test_chunk(text, split, text.len()),
    ];

    let adjusted = apply_kinsoku_start_rules(text, chunks);

    assert_eq!(adjusted.len(), 1);
    assert_eq!(adjusted[0].text, text);
    assert!(!adjusted[0].allow_glyph_fallback);
}

#[test]
fn merges_spacing_voicing_marks_with_previous_chunk() {
    for text in ["カ゛", "ハ゜"] {
        let split = text.chars().next().map(|ch| ch.len_utf8()).unwrap_or(0);
        let chunks = vec![
            test_chunk(text, 0, split),
            test_chunk(text, split, text.len()),
        ];

        let adjusted = apply_kinsoku_start_rules(text, chunks);

        assert_eq!(adjusted.len(), 1, "{text}");
        assert_eq!(adjusted[0].text, text, "{text}");
        assert!(!adjusted[0].allow_glyph_fallback, "{text}");
    }
}

#[test]
fn merges_jlreq_hyphens_with_previous_chunk() {
    for text in ["文‐", "文〜", "文゠", "文–"] {
        let split = "文".len();
        let chunks = vec![
            test_chunk(text, 0, split),
            test_chunk(text, split, text.len()),
        ];

        let adjusted = apply_kinsoku_start_rules(text, chunks);

        assert_eq!(adjusted.len(), 1, "{text}");
        assert_eq!(adjusted[0].text, text, "{text}");
        assert!(!adjusted[0].allow_glyph_fallback, "{text}");
    }
}

#[test]
fn merges_jlreq_inseparable_pairs() {
    for text in ["——", "……", "‥‥", "〳〵", "〴〵"] {
        let split = text.chars().next().map(|ch| ch.len_utf8()).unwrap_or(0);
        let chunks = vec![
            test_chunk(text, 0, split),
            test_chunk(text, split, text.len()),
        ];

        let adjusted = apply_kinsoku_start_rules(text, chunks);

        assert_eq!(adjusted.len(), 1, "{text}");
        assert_eq!(adjusted[0].text, text, "{text}");
        assert!(!adjusted[0].allow_glyph_fallback, "{text}");
    }
}

#[test]
fn keeps_different_jlreq_inseparable_classes_separate() {
    let text = "—…";
    let split = "—".len();
    let chunks = vec![
        test_chunk(text, 0, split),
        test_chunk(text, split, text.len()),
    ];

    let adjusted = apply_kinsoku_start_rules(text, chunks);

    assert_eq!(adjusted.len(), 2);
    assert_eq!(adjusted[0].text, "—");
    assert_eq!(adjusted[1].text, "…");
}

#[test]
fn marks_single_chunk_jlreq_inseparable_pair_unbreakable() {
    let text = "……";
    let chunks = vec![test_chunk(text, 0, text.len())];

    let adjusted = apply_kinsoku_start_rules(text, chunks);

    assert_eq!(adjusted.len(), 1);
    assert_eq!(adjusted[0].text, text);
    assert!(!adjusted[0].allow_glyph_fallback);
}

#[test]
fn merges_small_ka_ke_forbidden_line_start_with_previous_chunk() {
    for text in ["一ゕ", "一ゖ", "一ヵ", "一ヶ"] {
        let split = "一".len();
        let chunks = vec![
            test_chunk(text, 0, split),
            test_chunk(text, split, text.len()),
        ];

        let adjusted = apply_kinsoku_start_rules(text, chunks);

        assert_eq!(adjusted.len(), 1, "{text}");
        assert_eq!(adjusted[0].text, text, "{text}");
        assert!(!adjusted[0].allow_glyph_fallback, "{text}");
    }
}

#[test]
fn merges_katakana_phonetic_extension_small_kana_with_previous_chunk() {
    for text in [
        "一ㇰ", "一ㇱ", "一ㇲ", "一ㇳ", "一ㇴ", "一ㇵ", "一ㇶ", "一ㇷ", "一ㇸ", "一ㇹ", "一ㇺ",
        "一ㇻ", "一ㇼ", "一ㇽ", "一ㇾ", "一ㇿ",
    ] {
        let split = "一".len();
        let chunks = vec![
            test_chunk(text, 0, split),
            test_chunk(text, split, text.len()),
        ];

        let adjusted = apply_kinsoku_start_rules(text, chunks);

        assert_eq!(adjusted.len(), 1, "{text}");
        assert_eq!(adjusted[0].text, text, "{text}");
        assert!(!adjusted[0].allow_glyph_fallback, "{text}");
    }
}

#[test]
fn merges_cjk_white_closing_punctuation_with_previous_chunk() {
    for text in ["文〗", "文〙", "文〛", "文〟"] {
        let split = "文".len();
        let chunks = vec![
            test_chunk(text, 0, split),
            test_chunk(text, split, text.len()),
        ];

        let adjusted = apply_kinsoku_start_rules(text, chunks);

        assert_eq!(adjusted.len(), 1, "{text}");
        assert_eq!(adjusted[0].text, text, "{text}");
        assert!(!adjusted[0].allow_glyph_fallback, "{text}");
    }
}

#[test]
fn merges_cjk_double_prime_closing_quote_with_previous_chunk() {
    let text = "文〞";
    let split = "文".len();
    let chunks = vec![
        test_chunk(text, 0, split),
        test_chunk(text, split, text.len()),
    ];

    let adjusted = apply_kinsoku_start_rules(text, chunks);

    assert_eq!(adjusted.len(), 1);
    assert_eq!(adjusted[0].text, text);
    assert!(!adjusted[0].allow_glyph_fallback);
}

#[test]
fn merges_fullwidth_white_closing_parenthesis_with_previous_chunk() {
    let text = "文｠";
    let split = "文".len();
    let chunks = vec![
        test_chunk(text, 0, split),
        test_chunk(text, split, text.len()),
    ];

    let adjusted = apply_kinsoku_start_rules(text, chunks);

    assert_eq!(adjusted.len(), 1);
    assert_eq!(adjusted[0].text, text);
    assert!(!adjusted[0].allow_glyph_fallback);
}

#[test]
fn merges_prolonged_sound_mark_forbidden_line_start_with_previous_chunk() {
    let text = "カー";
    let split = "カ".len();
    let chunks = vec![
        test_chunk(text, 0, split),
        test_chunk(text, split, text.len()),
    ];

    let adjusted = apply_kinsoku_start_rules(text, chunks);

    assert_eq!(adjusted.len(), 1);
    assert_eq!(adjusted[0].text, text);
    assert!(!adjusted[0].allow_glyph_fallback);
}

#[test]
fn merges_iteration_mark_forbidden_line_start_with_previous_chunk() {
    let text = "時々";
    let split = "時".len();
    let chunks = vec![
        test_chunk(text, 0, split),
        test_chunk(text, split, text.len()),
    ];

    let adjusted = apply_kinsoku_start_rules(text, chunks);

    assert_eq!(adjusted.len(), 1);
    assert_eq!(adjusted[0].text, text);
    assert!(!adjusted[0].allow_glyph_fallback);
}

#[test]
fn merges_vertical_ideographic_iteration_mark_with_previous_chunk() {
    let text = "時〻";
    let split = "時".len();
    let chunks = vec![
        test_chunk(text, 0, split),
        test_chunk(text, split, text.len()),
    ];

    let adjusted = apply_kinsoku_start_rules(text, chunks);

    assert_eq!(adjusted.len(), 1);
    assert_eq!(adjusted[0].text, text);
    assert!(!adjusted[0].allow_glyph_fallback);
}

#[test]
fn keeps_halfwidth_open_corner_bracket_with_following_chunk() {
    let text = "｢中";
    let split = "｢".len();
    let chunks = vec![
        test_chunk(text, 0, split),
        test_chunk(text, split, text.len()),
    ];

    let adjusted = apply_kinsoku_start_rules(text, chunks);

    assert_eq!(adjusted.len(), 1);
    assert_eq!(adjusted[0].text, text);
    assert!(!adjusted[0].allow_glyph_fallback);
}

#[test]
fn keeps_cjk_white_opening_punctuation_with_following_chunk() {
    for text in ["〖文", "〘文", "〚文", "〝文"] {
        let split = text.chars().next().map(|ch| ch.len_utf8()).unwrap_or(0);
        let chunks = vec![
            test_chunk(text, 0, split),
            test_chunk(text, split, text.len()),
        ];

        let adjusted = apply_kinsoku_start_rules(text, chunks);

        assert_eq!(adjusted.len(), 1, "{text}");
        assert_eq!(adjusted[0].text, text, "{text}");
        assert!(!adjusted[0].allow_glyph_fallback, "{text}");
    }
}

#[test]
fn keeps_fullwidth_white_opening_parenthesis_with_following_chunk() {
    let text = "｟文";
    let split = "｟".len();
    let chunks = vec![
        test_chunk(text, 0, split),
        test_chunk(text, split, text.len()),
    ];

    let adjusted = apply_kinsoku_start_rules(text, chunks);

    assert_eq!(adjusted.len(), 1);
    assert_eq!(adjusted[0].text, text);
    assert!(!adjusted[0].allow_glyph_fallback);
}
