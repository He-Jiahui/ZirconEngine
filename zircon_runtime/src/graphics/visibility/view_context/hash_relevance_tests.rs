use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::time::{Duration, Instant};

use crate::core::framework::render::{
    CorePipelineKind, PrimitiveRelevance, RenderLayerSet, RenderMaterialAlphaMode,
};
use crate::core::framework::scene::Mobility;
use crate::graphics::visibility::declarations::VisibilityRelevanceEntry;

use super::relevance_by_stable_instance_key;

const BENCHMARK_ENTRY_COUNT: usize = 16_384;
const BENCHMARK_LOOKUP_COUNT: usize = 4_096;
const BENCHMARK_ITERATIONS: usize = 32;
const BENCHMARK_SAMPLES: usize = 17;

#[test]
fn runtime94_relevance_hash_index_preserves_latest_key_value() {
    let visible = PrimitiveRelevance::for_mesh_view(
        &RenderLayerSet::layer(2),
        CorePipelineKind::Core3d,
        &RenderLayerSet::layer(2),
        Mobility::Dynamic,
        RenderMaterialAlphaMode::Opaque,
    );
    let entries = [
        VisibilityRelevanceEntry {
            entity: 1,
            stable_instance_key: 41,
            relevance: PrimitiveRelevance::empty(),
        },
        VisibilityRelevanceEntry {
            entity: 2,
            stable_instance_key: 7,
            relevance: visible,
        },
        VisibilityRelevanceEntry {
            entity: 3,
            stable_instance_key: 41,
            relevance: visible,
        },
    ];

    let index = relevance_by_stable_instance_key(&entries);

    assert_eq!(index.len(), 2);
    assert_eq!(index.get(&7), Some(&visible));
    assert_eq!(index.get(&41), Some(&visible));
    assert_eq!(index.get(&99), None);
}

#[test]
fn runtime94_relevance_hash_index_keeps_bvh_output_order() {
    let source = include_str!("mod.rs");

    assert!(source.contains("use std::collections::{BTreeSet, HashMap}"));
    assert!(source.contains("fn relevance_by_stable_instance_key("));
    assert!(source.contains(".collect::<HashMap<_, _>>()"));
    assert!(source.contains("let entities = bvh_instances"));
    assert!(source.contains("let stable_instance_keys = bvh_instances"));
    assert!(source.contains("relevance_by_stable_instance_key"));
    assert!(!source.contains(".collect::<BTreeMap<_, _>>()"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn runtime94_relevance_hash_index_p95() {
    let entries = (0..BENCHMARK_ENTRY_COUNT as u64)
        .map(|index| {
            let key = index.wrapping_mul(11_400_714_819_323_198_485);
            (key, index.rotate_left(17))
        })
        .collect::<Vec<_>>();
    let lookup_keys = entries
        .iter()
        .rev()
        .take(BENCHMARK_LOOKUP_COUNT)
        .map(|(key, _)| *key)
        .collect::<Vec<_>>();
    let mut legacy_samples = Vec::with_capacity(BENCHMARK_SAMPLES);
    let mut optimized_samples = Vec::with_capacity(BENCHMARK_SAMPLES);

    for sample in 0..BENCHMARK_SAMPLES {
        if sample % 2 == 0 {
            legacy_samples.push(measure_legacy(&entries, &lookup_keys));
            optimized_samples.push(measure_optimized(&entries, &lookup_keys));
        } else {
            optimized_samples.push(measure_optimized(&entries, &lookup_keys));
            legacy_samples.push(measure_legacy(&entries, &lookup_keys));
        }
    }

    let legacy_p50 = percentile(&mut legacy_samples, 50);
    let legacy_p95 = percentile(&mut legacy_samples, 95);
    let optimized_p50 = percentile(&mut optimized_samples, 50);
    let optimized_p95 = percentile(&mut optimized_samples, 95);
    let reduction_basis_points = 10_000_u128.saturating_sub(
        optimized_p95.as_nanos().saturating_mul(10_000) / legacy_p95.as_nanos().max(1),
    );
    eprintln!(
        "RUNTIME94_RELEVANCE_HASH_INDEX_BENCH_V1 samples={BENCHMARK_SAMPLES} \
iterations={BENCHMARK_ITERATIONS} entries={BENCHMARK_ENTRY_COUNT} \
lookups={BENCHMARK_LOOKUP_COUNT} legacy_p50_ns={} legacy_p95_ns={} \
optimized_p50_ns={} optimized_p95_ns={} reduction_basis_points={reduction_basis_points}",
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        optimized_p50.as_nanos(),
        optimized_p95.as_nanos(),
    );
    assert!(
        optimized_p95.as_nanos().saturating_mul(100) <= legacy_p95.as_nanos().saturating_mul(70),
        "hash relevance indexing must reduce build-and-lookup P95 by at least 30%: \
legacy={legacy_p95:?}, optimized={optimized_p95:?}"
    );
}

fn measure_legacy(entries: &[(u64, u64)], lookup_keys: &[u64]) -> Duration {
    measure_index(entries, lookup_keys, |entries| {
        entries.iter().copied().collect::<BTreeMap<_, _>>()
    })
}

fn measure_optimized(entries: &[(u64, u64)], lookup_keys: &[u64]) -> Duration {
    measure_index(entries, lookup_keys, |entries| {
        entries.iter().copied().collect::<HashMap<_, _>>()
    })
}

fn measure_index<M>(
    entries: &[(u64, u64)],
    lookup_keys: &[u64],
    mut build: impl FnMut(&[(u64, u64)]) -> M,
) -> Duration
where
    M: LookupIndex,
{
    let started = Instant::now();
    let mut checksum = 0_u64;
    for _ in 0..BENCHMARK_ITERATIONS {
        let index = build(black_box(entries));
        for key in lookup_keys {
            checksum ^= index.value(black_box(key)).unwrap_or_default();
        }
        black_box(index);
    }
    black_box(checksum);
    started.elapsed()
}

trait LookupIndex {
    fn value(&self, key: &u64) -> Option<u64>;
}

impl LookupIndex for BTreeMap<u64, u64> {
    fn value(&self, key: &u64) -> Option<u64> {
        self.get(key).copied()
    }
}

impl LookupIndex for HashMap<u64, u64> {
    fn value(&self, key: &u64) -> Option<u64> {
        self.get(key).copied()
    }
}

fn percentile(samples: &mut [Duration], percentile: usize) -> Duration {
    samples.sort_unstable();
    let index = (samples.len() - 1).saturating_mul(percentile) / 100;
    samples[index]
}
