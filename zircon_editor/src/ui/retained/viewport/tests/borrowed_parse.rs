use std::hint::black_box;
use std::time::Instant;

use super::{parse_editor_hybrid_gi_profile, RenderHybridGiProfile};

const SAMPLE_PAIRS: usize = 21;
const PARSES_PER_SAMPLE: usize = 262_144;
const VALUE: &str = "  OPEN_WORLD  ";

#[test]
fn optimization_batch_20260826do_editor104_hybrid_gi_profile_preserves_tokens() {
    assert_eq!(
        parse_editor_hybrid_gi_profile("FULLY-DYNAMIC"),
        Some(RenderHybridGiProfile::FullyDynamic)
    );
    assert_eq!(
        parse_editor_hybrid_gi_profile(" indoor_static "),
        Some(RenderHybridGiProfile::IndoorStatic)
    );
    assert_eq!(
        parse_editor_hybrid_gi_profile("Open_World"),
        Some(RenderHybridGiProfile::OpenWorld)
    );
    assert_eq!(parse_editor_hybrid_gi_profile("open world"), None);
}

#[test]
fn optimization_batch_20260826do_editor104_hybrid_gi_profile_uses_borrowed_comparisons() {
    let source = include_str!("../editor_viewport_render_defaults.rs");
    assert!(source.contains("let value = value.trim();"));
    assert!(source.contains("value.eq_ignore_ascii_case(\"fully-dynamic\")"));
    assert!(source.contains("value.eq_ignore_ascii_case(\"open_world\")"));
    assert!(!source.contains("value.trim().to_ascii_lowercase().as_str()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826do_editor104_hybrid_gi_profile_borrowed_parse_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_parse));
            optimized_samples.push(measure(parse_editor_hybrid_gi_profile));
        } else {
            optimized_samples.push(measure(parse_editor_hybrid_gi_profile));
            legacy_samples.push(measure(legacy_parse));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR104_HYBRID_GI_PROFILE_BORROWED_PARSE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
parses_per_sample={PARSES_PER_SAMPLE} legacy_allocations_per_parse=1 \
optimized_allocations_per_parse=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed Hybrid GI profile parse P95 {optimized_p95_ns}ns must be at most 70% of allocated lowercase parse P95 {legacy_p95_ns}ns"
    );
}

fn legacy_parse(value: &str) -> Option<RenderHybridGiProfile> {
    match value.trim().to_ascii_lowercase().as_str() {
        "fully-dynamic" | "fully_dynamic" => Some(RenderHybridGiProfile::FullyDynamic),
        "indoor-static" | "indoor_static" => Some(RenderHybridGiProfile::IndoorStatic),
        "open-world" | "open_world" => Some(RenderHybridGiProfile::OpenWorld),
        "cinematic" => Some(RenderHybridGiProfile::Cinematic),
        "custom" => Some(RenderHybridGiProfile::Custom),
        _ => None,
    }
}

fn measure(parse: fn(&str) -> Option<RenderHybridGiProfile>) -> u128 {
    let started = Instant::now();
    let mut checksum = false;
    for _ in 0..PARSES_PER_SAMPLE {
        checksum ^= black_box(parse(black_box(VALUE))).is_some();
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
