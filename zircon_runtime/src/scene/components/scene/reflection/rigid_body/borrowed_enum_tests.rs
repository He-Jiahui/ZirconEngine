use std::hint::black_box;
use std::time::Instant;

use super::normalized_enum_name_matches;

const SAMPLE_PAIRS: usize = 21;
const MATCHES_PER_SAMPLE: usize = 262_144;
const VALUE: &str = "  AUTO-from__Shape  ";
const EXPECTED: &str = "autofromshape";

#[test]
fn optimization_batch_20260826do_runtime158_rigid_body_enum_match_preserves_normalization() {
    assert!(normalized_enum_name_matches("Explicit", "explicit"));
    assert!(normalized_enum_name_matches(
        " auto-from_shape ",
        "autofromshape"
    ));
    assert!(normalized_enum_name_matches("LINEAR CAST", "linearcast"));
    assert!(!normalized_enum_name_matches("nevermore", "never"));
    assert!(normalized_enum_name_matches("\u{e9}allow", "allow"));
}

#[test]
fn optimization_batch_20260826do_runtime158_rigid_body_enum_match_avoids_normalized_string() {
    let source = include_str!("../rigid_body.rs");
    assert_eq!(source.matches("normalized_enum_name_matches(").count(), 7);
    assert!(source.contains(".filter(u8::is_ascii_alphanumeric)"));
    assert!(source.contains(".eq(expected.bytes())"));
    assert!(!source.contains("fn normalized_enum_name(value: &str) -> String"));
    assert!(!source.contains("String::with_capacity(value.len())"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826do_runtime158_rigid_body_enum_borrowed_match_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_match));
            optimized_samples.push(measure(normalized_enum_name_matches));
        } else {
            optimized_samples.push(measure(normalized_enum_name_matches));
            legacy_samples.push(measure(legacy_match));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME158_RIGID_BODY_ENUM_BORROWED_MATCH_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
matches_per_sample={MATCHES_PER_SAMPLE} legacy_allocations_per_match=1 \
optimized_allocations_per_match=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed rigid-body enum match P95 {optimized_p95_ns}ns must be at most 70% of allocated normalization P95 {legacy_p95_ns}ns"
    );
}

fn legacy_match(value: &str, expected: &str) -> bool {
    let mut normalized = String::with_capacity(value.len());
    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_lowercase());
        }
    }
    normalized == expected
}

fn measure(matches: fn(&str, &str) -> bool) -> u128 {
    let started = Instant::now();
    let mut checksum = false;
    for _ in 0..MATCHES_PER_SAMPLE {
        checksum ^= black_box(matches(black_box(VALUE), black_box(EXPECTED)));
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
