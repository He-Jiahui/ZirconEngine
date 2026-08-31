use std::collections::VecDeque;
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::{
    CachedCircularProgressRaster, CircularProgressRasterCache, CircularProgressRasterKey,
    MAX_CIRCULAR_PROGRESS_RASTER_CACHE_ENTRIES,
};

const ENTRY_COUNT: usize = MAX_CIRCULAR_PROGRESS_RASTER_CACHE_ENTRIES;
const PAYLOAD_BYTES: usize = 64 * 1024;
const HITS_PER_FRAME: usize = ENTRY_COUNT;
const SAMPLE_COUNT: usize = 17;
const LEGACY_KEY_COMPARISONS: usize = ENTRY_COUNT * HITS_PER_FRAME;
const OPTIMIZED_HASH_LOOKUPS: usize = HITS_PER_FRAME;
const LEGACY_PIXEL_COPY_BYTES: usize = PAYLOAD_BYTES * HITS_PER_FRAME;

#[test]
fn optimization_batch_20260826bl_circular_progress_hash_arc_preserves_lru() {
    let mut cache = CircularProgressRasterCache::new();
    for index in 0..ENTRY_COUNT {
        cache.insert(key(index), format!("progress-{index}"), pixels(index));
    }

    let resident = Arc::clone(&cache.entries.get(&key(0)).unwrap().rgba);
    cache.access_generation = u64::MAX;
    let cached = cache.get(key(0)).expect("oldest entry is promoted");
    assert!(Arc::ptr_eq(&resident, &cached.rgba));
    assert_eq!(cache.access_generation, ENTRY_COUNT as u64 + 1);
    cache.insert(key(ENTRY_COUNT), "new".to_string(), pixels(ENTRY_COUNT));

    assert!(cache.entries.contains_key(&key(0)));
    assert!(!cache.entries.contains_key(&key(1)));
    assert!(cache.entries.contains_key(&key(ENTRY_COUNT)));
    assert_eq!(cache.entries.len(), ENTRY_COUNT);
}

#[test]
fn optimization_batch_20260826bl_circular_progress_hash_arc_eliminates_hit_copy() {
    const CACHE_SOURCE: &str = include_str!("../cache.rs");
    const ENTRY_SOURCE: &str = include_str!("../entry.rs");
    let production = CACHE_SOURCE.split("#[cfg(test)]").next().unwrap();

    assert_eq!(LEGACY_KEY_COMPARISONS, 16_384);
    assert_eq!(OPTIMIZED_HASH_LOOKUPS, 128);
    assert_eq!(LEGACY_PIXEL_COPY_BYTES, 8 * 1024 * 1024);
    assert!(production.contains("HashMap<CircularProgressRasterKey"));
    assert!(production.contains("rgba: Arc<[u8]>"));
    assert!(production.contains("self.entries.get_mut(&key)"));
    assert!(!production.contains("VecDeque<CircularProgressRasterCacheEntry>"));
    assert!(!production.contains(".iter().position"));
    assert!(ENTRY_SOURCE.contains("Arc::clone(&rgba)"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826bl_circular_progress_hash_arc_p95() {
    let hot_key = key(ENTRY_COUNT - 1);
    let mut legacy = VecDeque::new();
    let mut optimized = CircularProgressRasterCache::new();
    for index in 0..ENTRY_COUNT {
        legacy.push_back(LegacyEntry {
            key: key(index),
            resource_key: format!("progress-{index}"),
            rgba: vec![index as u8; PAYLOAD_BYTES],
        });
        optimized.insert(key(index), format!("progress-{index}"), pixels(index));
    }

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
        "PERF_RESULT EDITOR01_CIRCULAR_PROGRESS_HASH_ARC_CACHE_BENCH_V1 entries={ENTRY_COUNT} hits_per_frame={HITS_PER_FRAME} payload_bytes={PAYLOAD_BYTES} samples={SAMPLE_COUNT} sample_order=alternating legacy_key_comparisons={LEGACY_KEY_COMPARISONS} optimized_hash_lookups={OPTIMIZED_HASH_LOOKUPS} legacy_pixel_copy_bytes={LEGACY_PIXEL_COPY_BYTES} optimized_pixel_copy_bytes=0 deterministic_lookup_work_reduction_percent=99.2188 legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95 * 5 <= legacy_p95,
        "optimized P95 {optimized_p95}ns must be at least 80% below legacy P95 {legacy_p95}ns"
    );
}

struct LegacyEntry {
    key: CircularProgressRasterKey,
    resource_key: String,
    rgba: Vec<u8>,
}

fn legacy_frame(entries: &mut VecDeque<LegacyEntry>, hot_key: CircularProgressRasterKey) -> usize {
    let mut observed = 0;
    for _ in 0..HITS_PER_FRAME {
        let index = entries
            .iter()
            .position(|entry| entry.key == hot_key)
            .unwrap();
        let entry = entries.remove(index).unwrap();
        let cached = CachedCircularProgressRaster {
            resource_key: entry.resource_key.clone(),
            rgba: Arc::<[u8]>::from(entry.rgba.clone()),
        };
        observed += cached.resource_key.len() + cached.rgba.len();
        entries.push_back(entry);
    }
    observed
}

fn optimized_frame(
    cache: &mut CircularProgressRasterCache,
    hot_key: CircularProgressRasterKey,
) -> usize {
    let mut observed = 0;
    for _ in 0..HITS_PER_FRAME {
        let cached = cache.get(hot_key).unwrap();
        observed += cached.resource_key.len() + cached.rgba.len();
    }
    observed
}

fn key(index: usize) -> CircularProgressRasterKey {
    CircularProgressRasterKey::new(index as u32, 0.5, [1; 4], [2; 4])
}

fn pixels(index: usize) -> Arc<[u8]> {
    vec![index as u8; PAYLOAD_BYTES].into()
}

fn benchmark_paired_samples<const N: usize>(
    mut legacy: impl FnMut() -> usize,
    mut optimized: impl FnMut() -> usize,
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

fn measure(operation: &mut impl FnMut() -> usize) -> u128 {
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
