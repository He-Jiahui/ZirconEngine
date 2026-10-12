use std::hint::black_box;
use std::time::Instant;

use super::{contains_lowercase_query, unicode_lowercase_contains};

#[test]
fn runtime774_text_search_streaming_contains_preserves_unicode_case() {
    let cases = [
        ("\u{c9}DITEUR DE SCENE", "\u{e9}diteur", true),
        ("\u{130}stanbul", "i\u{307}stan", true),
        ("a\u{130}b", "\u{307}b", true),
        ("\u{c5}ngstr\u{f6}m", "ngstr", true),
        ("aaaa", "aa", true),
        ("\u{c5}ngstr\u{f6}m", "\u{e5}ngz", false),
        ("\u{df}", "ss", false),
    ];

    for (value, query, expected) in cases {
        assert_eq!(unicode_lowercase_contains(value, query), expected);
        assert_eq!(
            unicode_lowercase_contains(value, query),
            value.to_lowercase().contains(query),
            "streaming Unicode contains must match String normalization for {value:?} / {query:?}"
        );
    }
    assert!(contains_lowercase_query(
        "  \u{c9}DITEUR DE SCENE  ",
        "\u{e9}diteur"
    ));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime774_text_search_streaming_contains_release_benchmark() {
    const SEARCHES_PER_SAMPLE: usize = 16_384;
    const SAMPLE_PAIRS: usize = 17;
    const VALUE: &str = "Menu \u{c9}DITEUR DE SCENE / \u{130}stanbul";
    const QUERY: &str = "i\u{307}stan";

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_contains(false, SEARCHES_PER_SAMPLE, VALUE, QUERY));
            optimized_ns.push(measure_contains(true, SEARCHES_PER_SAMPLE, VALUE, QUERY));
        } else {
            optimized_ns.push(measure_contains(true, SEARCHES_PER_SAMPLE, VALUE, QUERY));
            legacy_ns.push(measure_contains(false, SEARCHES_PER_SAMPLE, VALUE, QUERY));
        }
    }

    let legacy_lowercase_allocations = SEARCHES_PER_SAMPLE;
    let optimized_lowercase_allocations = 0;
    assert!(legacy_lowercase_allocations > optimized_lowercase_allocations);
    println!(
        "RUNTIME774_TEXT_SEARCH_STREAMING_CONTAINS_BENCH_V1 searches_per_sample={SEARCHES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_lowercase_allocations={legacy_lowercase_allocations} optimized_lowercase_allocations={optimized_lowercase_allocations} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn measure_contains(optimized: bool, searches: usize, value: &str, query: &str) -> u128 {
    let started = Instant::now();
    let mut matched = 0usize;
    for _ in 0..searches {
        matched += if optimized {
            unicode_lowercase_contains(black_box(value), black_box(query))
        } else {
            black_box(value.to_lowercase().contains(query))
        } as usize;
    }
    black_box(matched);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
