use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::Instant;

const TOKEN_COUNT: usize = 65_536;
const UNIQUE_TOKEN_COUNT: usize = TOKEN_COUNT / 2;
const SAMPLE_COUNT: usize = 17;

fn tokens() -> Vec<u64> {
    (0..TOKEN_COUNT)
        .map(|index| ((index * 32_749) % UNIQUE_TOKEN_COUNT) as u64)
        .collect()
}

fn legacy_duplicate_count(tokens: &[u64]) -> usize {
    let mut seen = BTreeSet::new();
    tokens.iter().filter(|token| !seen.insert(**token)).count()
}

fn optimized_duplicate_count(tokens: &[u64]) -> usize {
    let mut seen = HashSet::with_capacity(tokens.len());
    tokens.iter().filter(|token| !seen.insert(**token)).count()
}

#[test]
fn optimization_batch_ir_editor628_watch_projection_hashes_seen_tokens_only() {
    let source = include_str!("../../watch_map.rs");
    let projection = source
        .split("pub fn project")
        .nth(1)
        .expect("watch projection production source")
        .split("fn project_canonical_dirty_tokens")
        .next()
        .expect("bounded non-canonical projection source");

    assert!(source.contains("use std::collections::{BTreeMap, BTreeSet, HashSet};"));
    assert!(projection.contains("HashSet::with_capacity(batch.dirty.len())"));
    assert!(projection.contains("let mut duplicates = BTreeSet::new()"));
    assert!(projection.contains("let mut unknown = BTreeSet::new()"));
    assert!(!projection.contains("let mut seen = BTreeSet::new()"));
}

#[test]
#[ignore = "Windows Release performance evidence; run through the validation coordinator"]
fn optimization_batch_ir_editor628_watch_seen_membership_performance_evidence() {
    let tokens = tokens();
    assert_eq!(
        legacy_duplicate_count(&tokens),
        TOKEN_COUNT - UNIQUE_TOKEN_COUNT
    );
    assert_eq!(
        legacy_duplicate_count(&tokens),
        optimized_duplicate_count(&tokens)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(legacy_duplicate_count(black_box(&tokens)));
            legacy_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(optimized_duplicate_count(black_box(&tokens)));
            optimized_samples.push(started.elapsed().as_nanos());
        } else {
            let started = Instant::now();
            black_box(optimized_duplicate_count(black_box(&tokens)));
            optimized_samples.push(started.elapsed().as_nanos());

            let started = Instant::now();
            black_box(legacy_duplicate_count(black_box(&tokens)));
            legacy_samples.push(started.elapsed().as_nanos());
        }
    }

    legacy_samples.sort_unstable();
    optimized_samples.sort_unstable();
    let legacy_p95 = legacy_samples[SAMPLE_COUNT - 1];
    let optimized_p95 = optimized_samples[SAMPLE_COUNT - 1];
    println!(
        "EDITOR628_HASH_WATCH_SEEN_MEMBERSHIP_BENCH_V1 tokens={TOKEN_COUNT} \
         unique_tokens={UNIQUE_TOKEN_COUNT} legacy_p95_ns={legacy_p95} \
         optimized_p95_ns={optimized_p95} target_ratio_bp=4000"
    );
    assert!(
        optimized_p95.saturating_mul(10_000) <= legacy_p95.saturating_mul(4_000),
        "hash watch membership P95 {optimized_p95} ns exceeded 40% of tree {legacy_p95} ns"
    );
}
