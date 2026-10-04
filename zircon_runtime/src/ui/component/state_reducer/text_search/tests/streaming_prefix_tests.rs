use std::hint::black_box;
use std::time::Instant;

use super::{starts_with_lowercase_query, unicode_lowercase_starts_with};

#[test]
fn runtime773_text_search_streaming_prefix_preserves_unicode_case() {
    assert!(starts_with_lowercase_query("  \u{c5}ngstrom", "\u{e5}ng"));
    assert!(unicode_lowercase_starts_with(
        "\u{130}stanbul",
        "i\u{307}stanbul"
    ));
    assert!(!unicode_lowercase_starts_with("\u{c5}ngstrom", "angstrom"));
    assert!(!unicode_lowercase_starts_with(
        "\u{c5}ngstrom",
        "\u{e5}meter"
    ));
    assert!(unicode_lowercase_starts_with(
        "already lowercase",
        "already"
    ));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn runtime773_text_search_streaming_prefix_release_benchmark() {
    const SEARCHES_PER_SAMPLE: usize = 16_384;
    const SAMPLE_PAIRS: usize = 17;
    const VALUE: &str = "\u{c5}ngstrom measurement label";
    const QUERY: &str = "\u{e5}ng";

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_prefix(false, SEARCHES_PER_SAMPLE, VALUE, QUERY));
            optimized_ns.push(measure_prefix(true, SEARCHES_PER_SAMPLE, VALUE, QUERY));
        } else {
            optimized_ns.push(measure_prefix(true, SEARCHES_PER_SAMPLE, VALUE, QUERY));
            legacy_ns.push(measure_prefix(false, SEARCHES_PER_SAMPLE, VALUE, QUERY));
        }
    }

    let legacy_lowercase_allocations = SEARCHES_PER_SAMPLE;
    let optimized_lowercase_allocations = 0;
    assert!(legacy_lowercase_allocations > optimized_lowercase_allocations);
    println!(
        "RUNTIME773_TEXT_SEARCH_STREAMING_PREFIX_BENCH_V1 searches_per_sample={SEARCHES_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS} legacy_lowercase_allocations={legacy_lowercase_allocations} optimized_lowercase_allocations={optimized_lowercase_allocations} legacy_p95_ns={} optimized_p95_ns={}",
        percentile(&legacy_ns, 95),
        percentile(&optimized_ns, 95),
    );
}

fn measure_prefix(optimized: bool, searches: usize, value: &str, query: &str) -> u128 {
    let started = Instant::now();
    let mut matched = 0usize;
    for _ in 0..searches {
        matched += if optimized {
            unicode_lowercase_starts_with(black_box(value), black_box(query))
        } else {
            black_box(value.to_lowercase().starts_with(query))
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
