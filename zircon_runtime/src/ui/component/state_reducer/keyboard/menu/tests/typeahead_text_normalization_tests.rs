use std::hint::black_box;
use std::time::Instant;

use super::keyboard_text_search;

#[test]
fn runtime775_menu_typeahead_text_normalization_preserves_legacy_output() {
    let cases = [
        ("", None),
        (" \n\t\u{0} ", None),
        ("  Open Project  ", Some("open project")),
        ("\u{2003}\u{c9}DITEUR\u{2003}", Some("\u{e9}diteur")),
        ("  \u{130}STANBUL\u{0}  ", Some("i\u{307}stanbul")),
        ("  Open\u{0} Project  ", Some("open project")),
        ("  a\u{0}\u{2003}b  ", Some("a\u{2003}b")),
    ];

    for (text, expected) in cases {
        assert_eq!(keyboard_text_search(text).as_deref(), expected);
        assert_eq!(
            keyboard_text_search(text),
            legacy_keyboard_text_search(text)
        );
    }
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime775_menu_typeahead_text_normalization_release_benchmark() {
    const KEY_EVENTS_PER_SAMPLE: usize = 16_384;
    const SAMPLE_PAIRS: usize = 17;
    const TEXT: &str = "  \u{c9}DITEUR \u{130}STANBUL\u{0}  ";

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_normalization(false, KEY_EVENTS_PER_SAMPLE, TEXT));
            optimized_ns.push(measure_normalization(true, KEY_EVENTS_PER_SAMPLE, TEXT));
        } else {
            optimized_ns.push(measure_normalization(true, KEY_EVENTS_PER_SAMPLE, TEXT));
            legacy_ns.push(measure_normalization(false, KEY_EVENTS_PER_SAMPLE, TEXT));
        }
    }

    let legacy_intermediate_string_buffers = KEY_EVENTS_PER_SAMPLE;
    let optimized_intermediate_string_buffers = 0;
    assert!(legacy_intermediate_string_buffers > optimized_intermediate_string_buffers);
    println!(
        "RUNTIME775_MENU_TYPEAHEAD_TEXT_NORMALIZATION_BENCH_V1 key_events_per_sample={KEY_EVENTS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_intermediate_string_buffers={legacy_intermediate_string_buffers} optimized_intermediate_string_buffers={optimized_intermediate_string_buffers} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn legacy_keyboard_text_search(text: &str) -> Option<String> {
    let search = text
        .chars()
        .filter(|ch| !ch.is_control())
        .collect::<String>()
        .trim()
        .to_lowercase();
    (!search.is_empty()).then_some(search)
}

fn measure_normalization(optimized: bool, key_events: usize, text: &str) -> u128 {
    let started = Instant::now();
    let mut normalized_bytes = 0usize;
    for _ in 0..key_events {
        let normalized = if optimized {
            keyboard_text_search(black_box(text))
        } else {
            legacy_keyboard_text_search(black_box(text))
        };
        normalized_bytes += normalized.as_deref().map_or(0, str::len);
    }
    black_box(normalized_bytes);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
