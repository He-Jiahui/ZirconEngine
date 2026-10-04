use super::{
    clamp_grapheme_boundary, leading_grapheme_continuation_len, line_end_boundary,
    line_start_boundary, next_grapheme_boundary, next_line_same_column_boundary,
    next_word_boundary, previous_grapheme_boundary, previous_line_same_column_boundary,
    previous_word_boundary, word_range_at,
};
use unicode_segmentation::UnicodeSegmentation;

#[test]
fn ascii_grapheme_joins_match_unicode_segmentation_for_every_byte_pair() {
    let mut joined = String::with_capacity(2);
    for previous in 0..=127_u8 {
        for next in 0..=127_u8 {
            joined.clear();
            joined.push(char::from(previous));
            joined.push(char::from(next));

            let first = joined.graphemes(true).next().expect("two ASCII bytes");
            let expected = usize::from(first.len() == 2);
            assert_eq!(
                leading_grapheme_continuation_len(&joined[..1], &joined[1..]),
                expected,
                "previous={previous:#04x}, next={next:#04x}"
            );
            assert_eq!(
                clamp_grapheme_boundary(&joined, 1),
                1 - expected,
                "clamp previous={previous:#04x}, next={next:#04x}"
            );
        }
    }
}

#[test]
fn ascii_grapheme_navigation_matches_segmentation_for_every_byte_pair() {
    let mut text = String::with_capacity(2);
    for first in 0..=127_u8 {
        for second in 0..=127_u8 {
            text.clear();
            text.push(char::from(first));
            text.push(char::from(second));
            for offset in 0..=text.len() {
                let expected_previous = text
                    .grapheme_indices(true)
                    .map(|(start, _)| start)
                    .take_while(|start| *start < offset)
                    .last();
                let expected_next = text
                    .grapheme_indices(true)
                    .map(|(start, grapheme)| start + grapheme.len())
                    .find(|end| *end > offset);
                assert_eq!(
                    previous_grapheme_boundary(&text, offset),
                    expected_previous,
                    "previous first={first:#04x}, second={second:#04x}, offset={offset}"
                );
                assert_eq!(
                    next_grapheme_boundary(&text, offset),
                    expected_next,
                    "next first={first:#04x}, second={second:#04x}, offset={offset}"
                );
            }
        }
    }
}

#[test]
fn grapheme_navigation_retains_mixed_unicode_context() {
    for text in [
        "\u{0600}ab\r\nc",
        "a\u{0301}bc",
        "\u{1f469}\u{200d}\u{1f680}ab",
        "x\r\ny\u{0301}z",
    ] {
        for offset in 0..=text.len() + 1 {
            let offset = (0..=offset.min(text.len()))
                .rev()
                .find(|candidate| text.is_char_boundary(*candidate))
                .expect("text start is a UTF-8 boundary");
            let expected_previous = text
                .grapheme_indices(true)
                .map(|(start, _)| start)
                .take_while(|start| *start < offset)
                .last();
            let expected_next = text
                .grapheme_indices(true)
                .map(|(start, grapheme)| start + grapheme.len())
                .find(|end| *end > offset);
            assert_eq!(previous_grapheme_boundary(text, offset), expected_previous);
            assert_eq!(next_grapheme_boundary(text, offset), expected_next);
        }
    }
}

#[test]
fn grapheme_join_retains_unicode_and_crlf_continuations() {
    assert_eq!(leading_grapheme_continuation_len("prefix\r", "\nsuffix"), 1);
    assert_eq!(leading_grapheme_continuation_len("prefix", "suffix"), 0);
    assert_eq!(
        leading_grapheme_continuation_len("a", "\u{0301}b"),
        "\u{0301}".len()
    );
    assert_eq!(
        leading_grapheme_continuation_len("\u{1f469}\u{200d}", "\u{1f680}x"),
        "\u{1f680}".len()
    );
}

#[test]
fn ascii_grapheme_boundary_clamp_preserves_mixed_unicode_context() {
    let text = "\u{1f469}\u{200d}\u{1f680}ab\r\ncd";
    let ascii_start = "\u{1f469}\u{200d}\u{1f680}".len();
    assert_eq!(
        clamp_grapheme_boundary(text, ascii_start + 1),
        ascii_start + 1
    );
    assert_eq!(
        clamp_grapheme_boundary(text, ascii_start + 3),
        ascii_start + 2
    );
    assert_eq!(clamp_grapheme_boundary(text, 5), 0);
}

#[test]
#[ignore = "release-only text layout performance evidence"]
fn ascii_grapheme_join_release_p95() {
    use std::{hint::black_box, time::Instant};

    const ITERATIONS: usize = 2_048;
    const SAMPLES: usize = 31;
    const MARKER: &str = "RUNTIME81_ASCII_GRAPHEME_JOIN_BENCH_V1";

    fn legacy(previous: &str, next: &str) -> usize {
        let split = previous.len();
        let mut combined = String::with_capacity(split + next.len());
        combined.push_str(previous);
        combined.push_str(next);
        for (start, grapheme) in combined.grapheme_indices(true) {
            let end = start + grapheme.len();
            if start < split && split < end {
                return end - split;
            }
            if start >= split {
                break;
            }
        }
        0
    }

    fn measure(join: fn(&str, &str) -> usize, previous: &str, next: &str) -> u128 {
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            black_box(join(black_box(previous), black_box(next)));
        }
        start.elapsed().as_nanos()
    }

    let previous = "ordinary text ".repeat(128);
    let next = "continued line";
    assert_eq!(
        legacy(&previous, next),
        leading_grapheme_continuation_len(&previous, next)
    );
    for _ in 0..5 {
        black_box(measure(legacy, &previous, next));
        black_box(measure(leading_grapheme_continuation_len, &previous, next));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut fast_samples = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        if sample % 2 == 0 {
            legacy_samples.push(measure(legacy, &previous, next));
            fast_samples.push(measure(leading_grapheme_continuation_len, &previous, next));
        } else {
            fast_samples.push(measure(leading_grapheme_continuation_len, &previous, next));
            legacy_samples.push(measure(legacy, &previous, next));
        }
    }
    let (legacy_p50, legacy_p95, legacy_p99) = ascii_bench_percentiles(&legacy_samples);
    let (fast_p50, fast_p95, fast_p99) = ascii_bench_percentiles(&fast_samples);
    println!(
        "{MARKER} previous_bytes={} next_bytes={} iterations={ITERATIONS} warmups=5 samples={SAMPLES} order=legacy_first_even_sample legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} fast_p50_ns={fast_p50} fast_p95_ns={fast_p95} fast_p99_ns={fast_p99} legacy_raw_ns={legacy_samples:?} fast_raw_ns={fast_samples:?} os={} arch={} package_version={}",
        previous.len(),
        next.len(),
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION")
    );
    assert!(
        fast_p95.saturating_mul(2) <= legacy_p95,
        "ASCII join P95 must be at least 50% faster"
    );
}

#[test]
#[ignore = "release-only text layout performance evidence"]
fn ascii_grapheme_boundary_release_p95() {
    use std::{hint::black_box, time::Instant};

    const ITERATIONS: usize = 2_048;
    const SAMPLES: usize = 31;
    const MARKER: &str = "RUNTIME81_ASCII_GRAPHEME_BOUNDARY_BENCH_V1";

    fn legacy(text: &str, offset: usize) -> usize {
        text.grapheme_indices(true)
            .map(|(index, _)| index)
            .take_while(|index| *index <= offset)
            .last()
            .unwrap_or(0)
    }

    fn measure(clamp: fn(&str, usize) -> usize, text: &str, offset: usize) -> u128 {
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            black_box(clamp(black_box(text), black_box(offset)));
        }
        start.elapsed().as_nanos()
    }

    let text = "ordinary text ".repeat(128);
    let offset = text.len() / 2;
    assert_eq!(
        legacy(&text, offset),
        clamp_grapheme_boundary(&text, offset)
    );
    for _ in 0..5 {
        black_box(measure(legacy, &text, offset));
        black_box(measure(clamp_grapheme_boundary, &text, offset));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut fast_samples = Vec::with_capacity(SAMPLES);
    for sample in 0..SAMPLES {
        if sample % 2 == 0 {
            legacy_samples.push(measure(legacy, &text, offset));
            fast_samples.push(measure(clamp_grapheme_boundary, &text, offset));
        } else {
            fast_samples.push(measure(clamp_grapheme_boundary, &text, offset));
            legacy_samples.push(measure(legacy, &text, offset));
        }
    }
    let (legacy_p50, legacy_p95, legacy_p99) = ascii_bench_percentiles(&legacy_samples);
    let (fast_p50, fast_p95, fast_p99) = ascii_bench_percentiles(&fast_samples);
    println!(
        "{MARKER} text_bytes={} offset={offset} iterations={ITERATIONS} warmups=5 samples={SAMPLES} order=legacy_first_even_sample legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} fast_p50_ns={fast_p50} fast_p95_ns={fast_p95} fast_p99_ns={fast_p99} legacy_raw_ns={legacy_samples:?} fast_raw_ns={fast_samples:?} os={} arch={} package_version={}",
        text.len(),
        std::env::consts::OS,
        std::env::consts::ARCH,
        env!("CARGO_PKG_VERSION")
    );
    assert!(
        fast_p95.saturating_mul(2) <= legacy_p95,
        "ASCII boundary clamp P95 must be at least 50% faster"
    );
}

#[test]
#[ignore = "release-only ASCII caret navigation performance evidence"]
fn ascii_grapheme_navigation_release_p95() {
    use std::{hint::black_box, time::Instant};

    const ITERATIONS: usize = 2_048;
    const SAMPLES: usize = 31;

    fn legacy_previous(text: &str, offset: usize) -> Option<usize> {
        text.grapheme_indices(true)
            .map(|(start, _)| start)
            .take_while(|start| *start < offset)
            .last()
    }

    fn legacy_next(text: &str, offset: usize) -> Option<usize> {
        text.grapheme_indices(true)
            .map(|(start, grapheme)| start + grapheme.len())
            .find(|end| *end > offset)
    }

    fn measure(navigation: fn(&str, usize) -> Option<usize>, text: &str, offset: usize) -> u128 {
        let start = Instant::now();
        for _ in 0..ITERATIONS {
            black_box(navigation(black_box(text), black_box(offset)));
        }
        start.elapsed().as_nanos()
    }

    let text = "ordinary text ".repeat(128);
    let offset = text.len() / 2;
    for (name, legacy, optimized) in [
        (
            "previous",
            legacy_previous as fn(&str, usize) -> Option<usize>,
            previous_grapheme_boundary as fn(&str, usize) -> Option<usize>,
        ),
        ("next", legacy_next, next_grapheme_boundary),
    ] {
        assert_eq!(legacy(&text, offset), optimized(&text, offset));
        for _ in 0..5 {
            black_box(measure(legacy, &text, offset));
            black_box(measure(optimized, &text, offset));
        }
        let mut legacy_samples = Vec::with_capacity(SAMPLES);
        let mut optimized_samples = Vec::with_capacity(SAMPLES);
        for sample in 0..SAMPLES {
            if sample % 2 == 0 {
                legacy_samples.push(measure(legacy, &text, offset));
                optimized_samples.push(measure(optimized, &text, offset));
            } else {
                optimized_samples.push(measure(optimized, &text, offset));
                legacy_samples.push(measure(legacy, &text, offset));
            }
        }
        let (legacy_p50, legacy_p95, legacy_p99) = ascii_bench_percentiles(&legacy_samples);
        let (optimized_p50, optimized_p95, optimized_p99) =
            ascii_bench_percentiles(&optimized_samples);
        println!(
            "RUNTIME81_ASCII_GRAPHEME_NAVIGATION_BENCH_V1 direction={name} text_bytes={} offset={offset} iterations={ITERATIONS} warmups=5 samples={SAMPLES} order=legacy_first_even_sample legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} legacy_p99_ns={legacy_p99} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} optimized_p99_ns={optimized_p99} legacy_raw_ns={legacy_samples:?} optimized_raw_ns={optimized_samples:?} os={} arch={} package_version={}",
            text.len(),
            std::env::consts::OS,
            std::env::consts::ARCH,
            env!("CARGO_PKG_VERSION")
        );
        assert!(
            optimized_p95.saturating_mul(2) <= legacy_p95,
            "ASCII caret navigation P95 must be at least 50% faster: {name}"
        );
    }
}

fn ascii_bench_percentiles(samples: &[u128]) -> (u128, u128, u128) {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = |percent: usize| sorted[(sorted.len() * percent).div_ceil(100) - 1];
    (rank(50), rank(95), rank(99))
}

#[test]
fn grapheme_boundary_floor_rejects_combining_and_zwj_interior_offsets() {
    let combining = "a\u{0301}b";
    assert_eq!(clamp_grapheme_boundary(combining, 1), 0);
    assert_eq!(clamp_grapheme_boundary(combining, 2), 0);
    assert_eq!(clamp_grapheme_boundary(combining, 3), 3);

    let zwj = "\u{1f469}\u{200d}\u{1f680}x";
    assert_eq!(clamp_grapheme_boundary(zwj, 4), 0);
    assert_eq!(clamp_grapheme_boundary(zwj, 7), 0);
    assert_eq!(clamp_grapheme_boundary(zwj, 11), 11);
}

#[test]
fn line_navigation_uses_every_canonical_hard_separator() {
    let text = "ab\u{2028}cd\u{0085}ef\u{2029}gh";
    let second_start = "ab\u{2028}".len();
    let third_start = "ab\u{2028}cd\u{0085}".len();
    let fourth_start = "ab\u{2028}cd\u{0085}ef\u{2029}".len();

    assert_eq!(line_start_boundary(text, third_start + 1), third_start);
    assert_eq!(
        line_end_boundary(text, second_start),
        third_start - "\u{0085}".len()
    );
    assert_eq!(
        previous_line_same_column_boundary(text, third_start + 1),
        Some(second_start + 1)
    );
    assert_eq!(
        next_line_same_column_boundary(text, third_start + 1),
        Some(fourth_start + 1)
    );
}

#[test]
fn line_navigation_keeps_crlf_as_one_separator() {
    let text = "ab\r\ncd\u{000b}ef\u{000c}gh";
    let second_start = "ab\r\n".len();
    let third_start = "ab\r\ncd\u{000b}".len();
    let fourth_start = "ab\r\ncd\u{000b}ef\u{000c}".len();

    assert_eq!(line_start_boundary(text, third_start + 1), third_start);
    assert_eq!(line_end_boundary(text, second_start), third_start - 1);
    assert_eq!(line_end_boundary(text, second_start - 1), second_start - 2);
    assert_eq!(
        previous_line_same_column_boundary(text, second_start + 1),
        Some(1)
    );
    assert_eq!(
        previous_line_same_column_boundary(text, third_start + 1),
        Some(second_start + 1)
    );
    assert_eq!(
        next_line_same_column_boundary(text, third_start + 1),
        Some(fourth_start + 1)
    );
}

#[test]
fn word_navigation_consumes_the_shared_unicode_boundary_owner() {
    let text = "alpha-beta can't";

    assert_eq!(previous_word_boundary(text, 8), Some(6));
    assert_eq!(next_word_boundary(text, 5), Some(10));
    assert_eq!(word_range_at(text, 12), Some((11, text.len())));
}
