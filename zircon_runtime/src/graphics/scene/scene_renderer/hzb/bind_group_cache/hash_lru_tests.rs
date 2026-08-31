use std::collections::{HashMap, VecDeque};
use std::hint::black_box;
use std::time::Instant;

use super::{HzbOcclusionBindGroupKey, MAX_HZB_OCCLUSION_BIND_GROUPS};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::MeshIndirectResourceIdentity;

const ENTRY_COUNT: usize = MAX_HZB_OCCLUSION_BIND_GROUPS;
const HITS_PER_FRAME: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const LEGACY_KEY_COMPARISONS: usize = ENTRY_COUNT * HITS_PER_FRAME;
const OPTIMIZED_HASH_LOOKUPS: usize = HITS_PER_FRAME;

#[test]
fn optimization_batch_20260826bn_hzb_bind_group_hash_lru_preserves_key_identity() {
    let mut entries = HashMap::new();
    entries.insert(key(7), "resident");

    assert_eq!(entries.get(&key(7)), Some(&"resident"));
    assert_ne!(key(7), key_with_revision(7, 2));
    assert_ne!(key(7), key(8));
}

#[test]
fn optimization_batch_20260826bn_hzb_bind_group_hash_lru_eliminates_linear_hit_scan() {
    const SOURCE: &str = include_str!("../bind_group_cache.rs");
    let production = SOURCE.split("#[cfg(test)]").next().unwrap();

    assert_eq!(LEGACY_KEY_COMPARISONS, 262_144);
    assert_eq!(OPTIMIZED_HASH_LOOKUPS, 4_096);
    assert!(production.contains("HashMap<HzbOcclusionBindGroupKey"));
    assert!(production.contains("last_used: u64"));
    assert!(production.contains("self.entries.get_mut(&key)"));
    assert!(production.contains("access_generation == u64::MAX"));
    assert!(!production.contains("VecDeque<HzbOcclusionBindGroupEntry>"));
    assert!(!production.contains(".iter().position"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826bn_hzb_bind_group_hash_lru_p95() {
    let mut legacy = (0..ENTRY_COUNT)
        .map(|index| LegacyEntry {
            key: key(index),
            last_used: index as u64,
        })
        .collect::<VecDeque<_>>();
    let mut optimized = (0..ENTRY_COUNT)
        .map(|index| (key(index), index as u64))
        .collect::<HashMap<_, _>>();
    let hot_key = key(ENTRY_COUNT - 1);
    let mut legacy_generation = ENTRY_COUNT as u64;
    let mut optimized_generation = ENTRY_COUNT as u64;

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || legacy_frame(black_box(&mut legacy), &mut legacy_generation, hot_key),
        || {
            optimized_frame(
                black_box(&mut optimized),
                &mut optimized_generation,
                hot_key,
            )
        },
    );
    assert_eq!(
        legacy_frame(&mut legacy, &mut legacy_generation, hot_key),
        optimized_frame(&mut optimized, &mut optimized_generation, hot_key)
    );

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT RUNTIME09B_HZB_BIND_GROUP_HASH_LRU_BENCH_V1 entries={ENTRY_COUNT} hits_per_frame={HITS_PER_FRAME} samples={SAMPLE_COUNT} sample_order=alternating legacy_key_comparisons={LEGACY_KEY_COMPARISONS} optimized_hash_lookups={OPTIMIZED_HASH_LOOKUPS} deterministic_lookup_work_reduction_percent=98.4375 legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95 * 2 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be at least 50% below legacy P95 {legacy_p95}ns"
    );
}

struct LegacyEntry {
    key: HzbOcclusionBindGroupKey,
    last_used: u64,
}

fn legacy_frame(
    entries: &mut VecDeque<LegacyEntry>,
    generation: &mut u64,
    hot_key: HzbOcclusionBindGroupKey,
) -> u64 {
    let mut observed = 0;
    for _ in 0..HITS_PER_FRAME {
        let index = entries
            .iter()
            .position(|entry| entry.key == hot_key)
            .unwrap();
        let mut entry = entries.remove(index).unwrap();
        *generation = generation.saturating_add(1);
        entry.last_used = *generation;
        observed += entry.key.sampled_resource_id;
        entries.push_back(entry);
    }
    observed
}

fn optimized_frame(
    entries: &mut HashMap<HzbOcclusionBindGroupKey, u64>,
    generation: &mut u64,
    hot_key: HzbOcclusionBindGroupKey,
) -> u64 {
    let mut observed = 0;
    for _ in 0..HITS_PER_FRAME {
        let last_used = entries.get_mut(&hot_key).unwrap();
        *generation = generation.saturating_add(1);
        *last_used = *generation;
        observed += hot_key.sampled_resource_id;
    }
    observed
}

fn key(index: usize) -> HzbOcclusionBindGroupKey {
    key_with_revision(index, 1)
}

fn key_with_revision(index: usize, revision: u64) -> HzbOcclusionBindGroupKey {
    HzbOcclusionBindGroupKey {
        sampled_resource_id: index as u64,
        indirect_resources: MeshIndirectResourceIdentity::new(index as u64, revision),
    }
}

fn benchmark_paired_samples<const N: usize>(
    mut legacy: impl FnMut() -> u64,
    mut optimized: impl FnMut() -> u64,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(N);
    let mut optimized_samples = Vec::with_capacity(N);
    for index in 0..N {
        if index % 2 == 0 {
            legacy_samples.push(measure(&mut legacy));
            optimized_samples.push(measure(&mut optimized));
        } else {
            optimized_samples.push(measure(&mut optimized));
            legacy_samples.push(measure(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn measure(operation: &mut impl FnMut() -> u64) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() - 1) * percentile / 100]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
