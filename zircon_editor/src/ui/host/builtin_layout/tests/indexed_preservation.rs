use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 31;
const BUILDS_PER_SAMPLE: usize = 16;
const OPEN_DESCRIPTOR_COUNT: usize = 1_024;

#[test]
fn optimization_batch_20260829an_editor259_indexed_preservation_matches_linear_scan() {
    let open = open_descriptors();
    let candidates = candidate_descriptors();

    assert_eq!(
        indexed_preserved(&open, &candidates),
        linear_preserved(&open, &candidates)
    );
    assert_eq!(indexed_preserved(&open, &[0, 512, 2_048]), 2);
}

#[test]
fn optimization_batch_20260829an_editor259_shell_restore_uses_open_descriptor_index() {
    let source = include_str!("../ensure_shell_instances.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("builtin shell production source");
    let preserved = production
        .split("fn preserved_single_instance")
        .nth(1)
        .expect("single-instance preservation")
        .split("fn restore_or_reuse_instance")
        .next()
        .expect("single-instance preservation body");

    assert!(production.contains("collect::<HashSet<_>>()"));
    assert!(preserved.contains("open_descriptor_ids.contains(&instance.descriptor_id)"));
    assert!(!preserved.contains("open_view_instances.values()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260829an_editor259_indexed_builtin_shell_preservation_bench() {
    let open = open_descriptors();
    let candidates = candidate_descriptors();
    assert_eq!(
        indexed_preserved(&open, &candidates),
        linear_preserved(&open, &candidates)
    );

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&open, &candidates, false));
            optimized_samples.push(measure(&open, &candidates, true));
        } else {
            optimized_samples.push(measure(&open, &candidates, true));
            legacy_samples.push(measure(&open, &candidates, false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR259_INDEXED_BUILTIN_SHELL_PRESERVATION_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} open_descriptors={OPEN_DESCRIPTOR_COUNT} \
candidates_per_build={OPEN_DESCRIPTOR_COUNT} legacy_worst_case_comparisons_per_build=1048576 \
optimized_hash_lookups_per_build=1024 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn open_descriptors() -> Vec<usize> {
    (0..OPEN_DESCRIPTOR_COUNT).collect()
}

fn candidate_descriptors() -> Vec<usize> {
    (OPEN_DESCRIPTOR_COUNT..OPEN_DESCRIPTOR_COUNT * 2).collect()
}

fn linear_preserved(open: &[usize], candidates: &[usize]) -> usize {
    candidates
        .iter()
        .filter(|candidate| open.iter().any(|current| current == *candidate))
        .count()
}

fn indexed_preserved(open: &[usize], candidates: &[usize]) -> usize {
    let open = open.iter().copied().collect::<HashSet<_>>();
    candidates
        .iter()
        .filter(|candidate| open.contains(*candidate))
        .count()
}

fn measure(open: &[usize], candidates: &[usize], optimized: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        checksum = checksum.wrapping_add(if optimized {
            indexed_preserved(black_box(open), black_box(candidates))
        } else {
            linear_preserved(black_box(open), black_box(candidates))
        });
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
