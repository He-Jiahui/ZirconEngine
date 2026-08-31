const NON_BREAKING_HYPHEN: char = '\u{2011}';
const NON_BREAKING_SPACE: char = '\u{00a0}';
const NARROW_NON_BREAKING_SPACE: char = '\u{202f}';
const WORD_JOINER: char = '\u{2060}';
const ZERO_WIDTH_NON_BREAKING_SPACE: char = '\u{feff}';
const ZERO_WIDTH_JOINER: char = '\u{200d}';
const VARIATION_SELECTOR_START: char = '\u{fe00}';
const VARIATION_SELECTOR_END: char = '\u{fe0f}';
const SUPPLEMENTARY_VARIATION_SELECTOR_START: char = '\u{e0100}';
const SUPPLEMENTARY_VARIATION_SELECTOR_END: char = '\u{e01ef}';

pub(super) fn allows_glyph_fallback(text: &str) -> bool {
    if text.is_ascii() {
        return true;
    }
    text.chars()
        .all(|character| !is_glue_character(character) && !is_variation_selector(character))
}

fn is_glue_character(ch: char) -> bool {
    matches!(
        ch,
        NON_BREAKING_HYPHEN
            | NON_BREAKING_SPACE
            | NARROW_NON_BREAKING_SPACE
            | WORD_JOINER
            | ZERO_WIDTH_NON_BREAKING_SPACE
            | ZERO_WIDTH_JOINER
    )
}

fn is_variation_selector(ch: char) -> bool {
    (VARIATION_SELECTOR_START..=VARIATION_SELECTOR_END).contains(&ch)
        || (SUPPLEMENTARY_VARIATION_SELECTOR_START..=SUPPLEMENTARY_VARIATION_SELECTOR_END)
            .contains(&ch)
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;
    use std::time::Instant;

    use super::allows_glyph_fallback;

    const SAMPLE_PAIRS: usize = 17;

    #[test]
    fn rejects_glyph_fallback_for_variation_selector_sequences() {
        assert!(!allows_glyph_fallback("✈\u{fe0f}"));
        assert!(!allows_glyph_fallback("禰\u{e0100}"));
    }

    #[test]
    fn rejects_glyph_fallback_for_additional_glue_characters() {
        for text in ["a\u{2011}b", "a\u{202f}b", "a\u{2060}b", "a\u{feff}b"] {
            assert!(!allows_glyph_fallback(text), "{text:?} must be glue");
        }
    }

    #[test]
    fn optimization_batch_dd_glue_scan_uses_ascii_fast_path_and_one_unicode_pass() {
        let source = include_str!("glue.rs")
            .split_once("#[cfg(test)]")
            .expect("production source and tests must remain separated")
            .0;
        assert!(source.contains("if text.is_ascii()"));
        assert!(source.contains("return true;"));
        assert!(source.contains(".all(|character|"));
        assert!(!source.contains(".any(is_glue_character) && !text.chars().any"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_dd_runtime410_glue_ascii_fast_path_p95() {
        const TEXT_LENGTH: usize = 65_536;
        let text = "a".repeat(TEXT_LENGTH);

        for _ in 0..3 {
            black_box(legacy_allows_glyph_fallback(black_box(&text)));
            black_box(allows_glyph_fallback(black_box(&text)));
        }

        let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
        let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
        for pair in 0..SAMPLE_PAIRS {
            if pair % 2 == 0 {
                legacy.push(measure(|| legacy_allows_glyph_fallback(&text)));
                optimized.push(measure(|| allows_glyph_fallback(&text)));
            } else {
                optimized.push(measure(|| allows_glyph_fallback(&text)));
                legacy.push(measure(|| legacy_allows_glyph_fallback(&text)));
            }
        }

        let legacy_p95_ns = percentile(&legacy, 95);
        let optimized_p95_ns = percentile(&optimized, 95);
        println!(
            "RUNTIME410_GLUE_ASCII_FAST_PATH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} text_length={TEXT_LENGTH} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
            csv(&legacy),
            csv(&optimized)
        );
        assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
    }

    fn legacy_allows_glyph_fallback(text: &str) -> bool {
        !text.chars().any(super::is_glue_character)
            && !text.chars().any(super::is_variation_selector)
    }

    fn measure(run: impl FnOnce() -> bool) -> u128 {
        let started = Instant::now();
        black_box(run());
        started.elapsed().as_nanos().max(1)
    }

    fn percentile(samples: &[u128], percentile: usize) -> u128 {
        let mut sorted = samples.to_vec();
        sorted.sort_unstable();
        sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
    }

    fn csv(samples: &[u128]) -> String {
        samples
            .iter()
            .map(u128::to_string)
            .collect::<Vec<_>>()
            .join(",")
    }
}
