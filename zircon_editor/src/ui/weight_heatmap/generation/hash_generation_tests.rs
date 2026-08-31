use std::collections::{BTreeMap, VecDeque};
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::{
    StaticFieldCache, StaticFieldCacheKey, WeightHeatmapStaticField, STATIC_FIELD_CACHE_CAPACITY,
};

const ENTRY_COUNT: usize = STATIC_FIELD_CACHE_CAPACITY;
const HITS_PER_FRAME: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const LEGACY_RECENCY_COMPARISONS: usize = ENTRY_COUNT * HITS_PER_FRAME;
const OPTIMIZED_HASH_LOOKUPS: usize = HITS_PER_FRAME;

#[test]
fn optimization_batch_20260826bn_weight_heatmap_hash_generation_preserves_lru() {
    let mut cache = StaticFieldCache::default();
    for index in 0..ENTRY_COUNT {
        cache.insert_or_get(key(index), field(index));
    }

    let resident = Arc::clone(&cache.entries.get(&key(0)).unwrap().field);
    cache.access_generation = u64::MAX;
    let cached = cache.get(key(0)).expect("oldest field is promoted");
    assert!(Arc::ptr_eq(&resident, &cached));
    assert_eq!(cache.access_generation, ENTRY_COUNT as u64 + 1);
    cache.insert_or_get(key(ENTRY_COUNT), field(ENTRY_COUNT));

    assert!(cache.entries.contains_key(&key(0)));
    assert!(!cache.entries.contains_key(&key(1)));
    assert!(cache.entries.contains_key(&key(ENTRY_COUNT)));
    assert_eq!(cache.entries.len(), ENTRY_COUNT);
}

#[test]
fn optimization_batch_20260826bn_weight_heatmap_hash_generation_eliminates_recency_scan() {
    const SOURCE: &str = include_str!("../generation.rs");
    let production = SOURCE.split("#[cfg(test)]").next().unwrap();

    assert_eq!(LEGACY_RECENCY_COMPARISONS, 65_536);
    assert_eq!(OPTIMIZED_HASH_LOOKUPS, 4_096);
    assert!(production.contains("HashMap<StaticFieldCacheKey, StaticFieldCacheEntry>"));
    assert!(production.contains("last_used: u64"));
    assert!(production.contains("self.entries.get_mut(&key)"));
    assert!(production.contains("access_generation != u64::MAX"));
    assert!(!production.contains("VecDeque<StaticFieldCacheKey>"));
    assert!(!production.contains("self.recency.retain"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826bn_weight_heatmap_hash_generation_p95() {
    let mut legacy = LegacyCache::default();
    let mut optimized = StaticFieldCache::default();
    for index in 0..ENTRY_COUNT {
        legacy.insert(key(index), field(index));
        optimized.insert_or_get(key(index), field(index));
    }
    let hot_key = key(ENTRY_COUNT - 1);

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || legacy_frame(black_box(&mut legacy), hot_key),
        || optimized_frame(black_box(&mut optimized), hot_key),
    );
    assert_eq!(
        legacy_frame(&mut legacy, hot_key),
        optimized_frame(&mut optimized, hot_key)
    );

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT EDITOR01_WEIGHT_HEATMAP_HASH_GENERATION_CACHE_BENCH_V1 entries={ENTRY_COUNT} hits_per_frame={HITS_PER_FRAME} samples={SAMPLE_COUNT} sample_order=alternating legacy_recency_comparisons={LEGACY_RECENCY_COMPARISONS} optimized_recency_comparisons=0 optimized_hash_lookups={OPTIMIZED_HASH_LOOKUPS} deterministic_recency_work_reduction_percent=100.0000 legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95 * 2 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be at least 50% below legacy P95 {legacy_p95}ns"
    );
}

#[derive(Default)]
struct LegacyCache {
    entries: BTreeMap<StaticFieldCacheKey, Arc<WeightHeatmapStaticField>>,
    recency: VecDeque<StaticFieldCacheKey>,
}

impl LegacyCache {
    fn insert(&mut self, key: StaticFieldCacheKey, field: Arc<WeightHeatmapStaticField>) {
        self.entries.insert(key, field);
        self.touch(key);
    }

    fn get(&mut self, key: StaticFieldCacheKey) -> Option<Arc<WeightHeatmapStaticField>> {
        let field = self.entries.get(&key).cloned();
        if field.is_some() {
            self.touch(key);
        }
        field
    }

    fn touch(&mut self, key: StaticFieldCacheKey) {
        self.recency.retain(|entry| *entry != key);
        self.recency.push_back(key);
    }
}

fn legacy_frame(cache: &mut LegacyCache, hot_key: StaticFieldCacheKey) -> u64 {
    let mut observed = 0;
    for _ in 0..HITS_PER_FRAME {
        observed += cache.get(hot_key).unwrap().generation();
    }
    observed
}

fn optimized_frame(cache: &mut StaticFieldCache, hot_key: StaticFieldCacheKey) -> u64 {
    let mut observed = 0;
    for _ in 0..HITS_PER_FRAME {
        observed += cache.get(hot_key).unwrap().generation();
    }
    observed
}

fn key(index: usize) -> StaticFieldCacheKey {
    StaticFieldCacheKey {
        static_generation: index as u64,
        columns: 1,
        rows: 1,
    }
}

fn field(index: usize) -> Arc<WeightHeatmapStaticField> {
    Arc::new(WeightHeatmapStaticField {
        columns: 1,
        rows: 1,
        intensities: vec![index as f32].into(),
        generation: index as u64,
    })
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
