use std::hint::black_box;
use std::time::Instant;

use super::normalize_windows_path_identity;

const SAMPLE_PAIRS: usize = 21;
const IDENTITIES_PER_SAMPLE: usize = 16_384;

#[test]
fn optimization_batch_20260826dp_runtime159_migration_path_identity_preserves_ascii_case_fold() {
    assert_eq!(
        normalize_windows_path_identity("C:\\Project\\ASSETS\\Hero.ZMETA".to_string()),
        "c:\\project\\assets\\hero.zmeta"
    );
    assert_eq!(
        normalize_windows_path_identity("C:\\Project\\\u{e9}Hero".to_string()),
        "c:\\project\\\u{e9}hero"
    );
}

#[test]
fn optimization_batch_20260826dp_runtime159_migration_path_identity_reuses_owned_buffer() {
    let mut identity = String::with_capacity(256);
    identity.push_str("C:\\PROJECT\\ASSETS\\MATERIALS\\HERO.ZMETA");
    let allocation = identity.as_ptr();
    let capacity = identity.capacity();

    let normalized = normalize_windows_path_identity(identity);
    assert_eq!(normalized.as_ptr(), allocation);
    assert_eq!(normalized.capacity(), capacity);
    assert_eq!(normalized, "c:\\project\\assets\\materials\\hero.zmeta");

    let source = include_str!("../recovery.rs");
    assert!(source.contains("identity.make_ascii_lowercase();"));
    assert!(source.contains("resolved.to_string_lossy().into_owned()"));
    assert!(!source.contains("resolved.to_string_lossy().to_ascii_lowercase()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826dp_runtime159_migration_path_identity_in_place_lowercase_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        let legacy = fixture_identities();
        let optimized = legacy.clone();
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy, legacy_normalize));
            optimized_samples.push(measure(optimized, normalize_windows_path_identity));
        } else {
            optimized_samples.push(measure(optimized, normalize_windows_path_identity));
            legacy_samples.push(measure(legacy, legacy_normalize));
        }
    }

    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME159_MIGRATION_PATH_IDENTITY_IN_PLACE_LOWERCASE_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
identities_per_sample={IDENTITIES_PER_SAMPLE} legacy_lowercase_allocations_per_sample={IDENTITIES_PER_SAMPLE} \
optimized_lowercase_allocations_per_sample=0 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "in-place migration path lowercase P95 {optimized_p95_ns}ns must be at most 70% of allocated lowercase P95 {legacy_p95_ns}ns"
    );
}

fn fixture_identities() -> Vec<String> {
    (0..IDENTITIES_PER_SAMPLE)
        .map(|index| {
            format!("C:\\PROJECT\\CONTENT\\MATERIALS\\CHARACTERS\\HERO_{index:05}\\SURFACE.ZMETA")
        })
        .collect()
}

fn legacy_normalize(identity: String) -> String {
    identity.to_ascii_lowercase()
}

fn measure(identities: Vec<String>, normalize: fn(String) -> String) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for identity in identities {
        checksum ^= black_box(normalize(black_box(identity))).len();
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
