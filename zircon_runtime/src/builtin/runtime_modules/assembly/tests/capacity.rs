use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 21;
const REGISTRIES_PER_SAMPLE: usize = 64;
const CANDIDATES_PER_REGISTRY: usize = 4_096;

#[test]
fn optimization_batch_20260826gl_runtime232_candidate_registry_reserves_both_indices() {
    let registry = BuiltinModuleCandidateRegistry::with_capacity(CANDIDATES_PER_REGISTRY);

    assert!(registry.candidates.capacity() >= CANDIDATES_PER_REGISTRY);
    assert!(registry.index_by_id.capacity() >= CANDIDATES_PER_REGISTRY);
    assert!(registry.candidates.is_empty());
    assert!(registry.index_by_id.is_empty());
}

#[test]
fn optimization_batch_20260826gl_runtime232_profile_selection_uses_module_count_capacity() {
    let source = include_str!("../profile_selection.rs");

    assert!(source.contains("let mut registry = Self::with_capacity(modules.len());"));
    assert!(source.contains("candidates: Vec::with_capacity(candidate_count)"));
    assert!(source.contains("index_by_id: HashMap::with_capacity(candidate_count)"));
    assert!(!source.contains("candidates: Vec::new()"));
    assert!(!source.contains("index_by_id: HashMap::new()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gl_runtime232_builtin_candidate_registry_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME232_BUILTIN_CANDIDATE_REGISTRY_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
registries_per_sample={REGISTRIES_PER_SAMPLE} candidates_per_registry={CANDIDATES_PER_REGISTRY} \
legacy_preallocated_containers_per_registry=0 optimized_preallocated_containers_per_registry=2 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for registry in 0..REGISTRIES_PER_SAMPLE {
        let mut candidates = if reserve {
            Vec::with_capacity(CANDIDATES_PER_REGISTRY)
        } else {
            Vec::new()
        };
        let mut index_by_id = if reserve {
            HashMap::with_capacity(CANDIDATES_PER_REGISTRY)
        } else {
            HashMap::new()
        };
        for candidate in 0..CANDIDATES_PER_REGISTRY {
            let id = black_box(registry * CANDIDATES_PER_REGISTRY + candidate);
            index_by_id.insert(id, candidates.len());
            candidates.push((id, id.rotate_left(7)));
        }
        checksum ^= black_box(candidates.capacity() ^ index_by_id.capacity());
        black_box((&candidates, &index_by_id));
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
